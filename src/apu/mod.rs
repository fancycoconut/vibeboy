const CPU_FREQ: f64 = 4_194_304.0;
pub const SAMPLE_RATE: u32 = 44_100;
const CYCLES_PER_SAMPLE: f64 = CPU_FREQ / SAMPLE_RATE as f64;
const FS_PERIOD: u32 = 8192; // T-cycles per frame sequencer step (512 Hz)

const DUTY: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1], // 12.5%
    [1, 0, 0, 0, 0, 0, 0, 1], // 25%
    [1, 0, 0, 0, 0, 1, 1, 1], // 50%
    [0, 1, 1, 1, 1, 1, 1, 0], // 75%
];

fn noise_divisor(code: u8) -> u32 {
    [8u32, 16, 32, 48, 64, 80, 96, 112][(code & 7) as usize]
}

// Shared square-wave channel state (used by CH1 and CH2)
struct SquareCh {
    duty: u8,
    length_counter: u8,
    volume_init: u8,
    env_add: bool,
    env_period: u8,
    freq: u16,
    length_enable: bool,

    enabled: bool,
    dac_on: bool,
    volume: u8,
    env_timer: u8,
    freq_timer: u32, // T-cycles until next duty step
    duty_pos: u8,
}

impl SquareCh {
    fn new() -> Self {
        Self {
            duty: 0,
            length_counter: 0,
            volume_init: 0,
            env_add: false,
            env_period: 0,
            freq: 0,
            length_enable: false,
            enabled: false,
            dac_on: false,
            volume: 0,
            env_timer: 0,
            freq_timer: 8192,
            duty_pos: 0,
        }
    }

    fn freq_reload(&self) -> u32 {
        (2048u32.saturating_sub(self.freq as u32)) * 4
    }

    fn tick(&mut self, tcycles: u32) {
        if !self.enabled {
            return;
        }
        let reload = self.freq_reload();
        if reload == 0 {
            return;
        }
        let mut rem = tcycles;
        while rem > 0 {
            if rem >= self.freq_timer {
                rem -= self.freq_timer;
                self.freq_timer = reload;
                self.duty_pos = (self.duty_pos + 1) & 7;
            } else {
                self.freq_timer -= rem;
                rem = 0;
            }
        }
    }

    fn sample(&self) -> u8 {
        if !self.enabled || !self.dac_on {
            return 0;
        }
        if DUTY[self.duty as usize][self.duty_pos as usize] != 0 {
            self.volume
        } else {
            0
        }
    }

    fn trigger(&mut self) {
        self.enabled = true;
        if self.length_counter == 0 {
            self.length_counter = 64;
        }
        self.freq_timer = self.freq_reload();
        self.env_timer = if self.env_period == 0 { 8 } else { self.env_period };
        self.volume = self.volume_init;
        if !self.dac_on {
            self.enabled = false;
        }
    }

    fn step_length(&mut self) {
        if self.length_enable && self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 {
                self.enabled = false;
            }
        }
    }

    fn step_envelope(&mut self) {
        if self.env_period == 0 {
            return;
        }
        self.env_timer = self.env_timer.saturating_sub(1);
        if self.env_timer == 0 {
            self.env_timer = self.env_period;
            if self.env_add && self.volume < 15 {
                self.volume += 1;
            } else if !self.env_add && self.volume > 0 {
                self.volume -= 1;
            }
        }
    }
}

// CH1: square wave with frequency sweep
struct Ch1 {
    sq: SquareCh,
    sweep_period: u8,
    sweep_negate: bool,
    sweep_shift: u8,
    sweep_timer: u8,
    sweep_enabled: bool,
    shadow_freq: u16,
}

impl Ch1 {
    fn new() -> Self {
        Self {
            sq: SquareCh::new(),
            sweep_period: 0,
            sweep_negate: false,
            sweep_shift: 0,
            sweep_timer: 8,
            sweep_enabled: false,
            shadow_freq: 0,
        }
    }

    fn calc_sweep_freq(&self) -> u16 {
        let shifted = self.shadow_freq >> self.sweep_shift;
        if self.sweep_negate {
            self.shadow_freq.saturating_sub(shifted)
        } else {
            self.shadow_freq.saturating_add(shifted)
        }
    }

    fn trigger(&mut self) {
        self.sq.trigger();
        self.shadow_freq = self.sq.freq;
        self.sweep_timer = if self.sweep_period == 0 { 8 } else { self.sweep_period };
        self.sweep_enabled = self.sweep_period != 0 || self.sweep_shift != 0;
        if self.sweep_shift != 0 && self.calc_sweep_freq() > 2047 {
            self.sq.enabled = false;
        }
    }

    fn step_sweep(&mut self) {
        if self.sweep_timer > 0 {
            self.sweep_timer -= 1;
        }
        if self.sweep_timer == 0 {
            self.sweep_timer = if self.sweep_period == 0 { 8 } else { self.sweep_period };
            if self.sweep_enabled && self.sweep_period != 0 {
                let new_freq = self.calc_sweep_freq();
                if new_freq > 2047 {
                    self.sq.enabled = false;
                } else {
                    self.shadow_freq = new_freq;
                    self.sq.freq = new_freq;
                    // Second overflow check after writing back
                    if self.calc_sweep_freq() > 2047 {
                        self.sq.enabled = false;
                    }
                }
            }
        }
    }
}

// CH3: wave channel — plays 4-bit samples from wave RAM
struct Ch3 {
    dac_on: bool,
    enabled: bool,
    length_counter: u16,
    volume_code: u8, // 0=mute, 1=100%, 2=50%, 3=25%
    freq: u16,
    length_enable: bool,
    freq_timer: u32,
    wave_pos: u8,    // 0-31 (index into 32 4-bit samples)
    wave_ram: [u8; 16],
}

impl Ch3 {
    fn new() -> Self {
        Self {
            dac_on: false,
            enabled: false,
            length_counter: 0,
            volume_code: 0,
            freq: 0,
            length_enable: false,
            freq_timer: 4096,
            wave_pos: 0,
            wave_ram: [0; 16],
        }
    }

    fn freq_reload(&self) -> u32 {
        (2048u32.saturating_sub(self.freq as u32)) * 2
    }

    fn tick(&mut self, tcycles: u32) {
        if !self.enabled {
            return;
        }
        let reload = self.freq_reload();
        if reload == 0 {
            return;
        }
        let mut rem = tcycles;
        while rem > 0 {
            if rem >= self.freq_timer {
                rem -= self.freq_timer;
                self.freq_timer = reload;
                self.wave_pos = (self.wave_pos + 1) & 31;
            } else {
                self.freq_timer -= rem;
                rem = 0;
            }
        }
    }

    fn sample(&self) -> u8 {
        if !self.enabled || !self.dac_on {
            return 0;
        }
        let byte = self.wave_ram[(self.wave_pos / 2) as usize];
        let nibble = if self.wave_pos & 1 == 0 { byte >> 4 } else { byte & 0x0F };
        match self.volume_code & 3 {
            1 => nibble,
            2 => nibble >> 1,
            3 => nibble >> 2,
            _ => 0,
        }
    }

    fn trigger(&mut self) {
        self.enabled = true;
        if self.length_counter == 0 {
            self.length_counter = 256;
        }
        self.freq_timer = self.freq_reload();
        self.wave_pos = 0;
        if !self.dac_on {
            self.enabled = false;
        }
    }

    fn step_length(&mut self) {
        if self.length_enable && self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 {
                self.enabled = false;
            }
        }
    }
}

// CH4: noise channel with LFSR
struct Ch4 {
    enabled: bool,
    dac_on: bool,
    length_counter: u8,
    volume_init: u8,
    env_add: bool,
    env_period: u8,
    clock_shift: u8,
    width_mode: bool, // true = 7-bit LFSR, false = 15-bit
    divisor_code: u8,
    length_enable: bool,

    volume: u8,
    env_timer: u8,
    freq_timer: u32,
    lfsr: u16,
}

impl Ch4 {
    fn new() -> Self {
        Self {
            enabled: false,
            dac_on: false,
            length_counter: 0,
            volume_init: 0,
            env_add: false,
            env_period: 0,
            clock_shift: 0,
            width_mode: false,
            divisor_code: 0,
            length_enable: false,
            volume: 0,
            env_timer: 8,
            freq_timer: 8,
            lfsr: 0x7FFF,
        }
    }

    fn freq_reload(&self) -> u32 {
        noise_divisor(self.divisor_code) << self.clock_shift
    }

    fn tick(&mut self, tcycles: u32) {
        if !self.enabled {
            return;
        }
        let reload = self.freq_reload();
        if reload == 0 {
            return;
        }
        let mut rem = tcycles;
        while rem > 0 {
            if rem >= self.freq_timer {
                rem -= self.freq_timer;
                self.freq_timer = reload;
                self.clock_lfsr();
            } else {
                self.freq_timer -= rem;
                rem = 0;
            }
        }
    }

    fn clock_lfsr(&mut self) {
        let xor = (self.lfsr & 1) ^ ((self.lfsr >> 1) & 1);
        self.lfsr = (self.lfsr >> 1) | (xor << 14);
        if self.width_mode {
            self.lfsr = (self.lfsr & !(1 << 6)) | (xor << 6);
        }
    }

    fn sample(&self) -> u8 {
        if !self.enabled || !self.dac_on {
            return 0;
        }
        // LFSR bit 0 low = output HIGH
        if self.lfsr & 1 == 0 { self.volume } else { 0 }
    }

    fn trigger(&mut self) {
        self.enabled = true;
        if self.length_counter == 0 {
            self.length_counter = 64;
        }
        self.freq_timer = self.freq_reload();
        self.env_timer = if self.env_period == 0 { 8 } else { self.env_period };
        self.volume = self.volume_init;
        self.lfsr = 0x7FFF;
        if !self.dac_on {
            self.enabled = false;
        }
    }

    fn step_length(&mut self) {
        if self.length_enable && self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 {
                self.enabled = false;
            }
        }
    }

    fn step_envelope(&mut self) {
        if self.env_period == 0 {
            return;
        }
        self.env_timer = self.env_timer.saturating_sub(1);
        if self.env_timer == 0 {
            self.env_timer = self.env_period;
            if self.env_add && self.volume < 15 {
                self.volume += 1;
            } else if !self.env_add && self.volume > 0 {
                self.volume -= 1;
            }
        }
    }
}

pub struct Apu {
    ch1: Ch1,
    ch2: SquareCh,
    ch3: Ch3,
    ch4: Ch4,

    nr50: u8,
    nr51: u8,
    power: bool,

    fs_counter: u32, // T-cycles until next frame sequencer step
    fs_step: u8,     // 0-7

    sample_acc: f64,

    // High-pass filter capacitors (removes DC offset)
    hp_cap_l: f32,
    hp_cap_r: f32,

    pub samples: Vec<f32>, // Stereo interleaved (L, R, L, R, ...)
}

impl Apu {
    pub fn new() -> Self {
        let mut apu = Self {
            ch1: Ch1::new(),
            ch2: SquareCh::new(),
            ch3: Ch3::new(),
            ch4: Ch4::new(),
            nr50: 0,
            nr51: 0,
            power: false,
            fs_counter: FS_PERIOD,
            fs_step: 0,
            sample_acc: 0.0,
            hp_cap_l: 0.0,
            hp_cap_r: 0.0,
            samples: Vec::with_capacity(2048),
        };
        // Post-boot-ROM APU state (skipping boot ROM)
        apu.write(0xFF26, 0xF1); // NR52: power on
        apu.write(0xFF11, 0xBF); // NR11: duty=10 50%, length=63
        apu.write(0xFF12, 0xF3); // NR12: vol=15, decay, period=3
        apu.write(0xFF25, 0xF3); // NR51: CH1+CH2 on both, CH3/4 on left
        apu.write(0xFF24, 0x77); // NR50: full volume both sides
        apu
    }

    pub fn step(&mut self, cycles: u8) {
        if !self.power {
            return;
        }

        let tc = cycles as u32 * 4;

        self.ch1.sq.tick(tc);
        self.ch2.tick(tc);
        self.ch3.tick(tc);
        self.ch4.tick(tc);

        // Advance frame sequencer
        if tc < self.fs_counter {
            self.fs_counter -= tc;
        } else {
            let overshoot = tc - self.fs_counter;
            self.fs_counter = FS_PERIOD - (overshoot % FS_PERIOD);
            let extra_steps = 1 + overshoot / FS_PERIOD;
            for _ in 0..extra_steps {
                self.tick_frame_sequencer();
            }
        }

        // Generate audio samples
        self.sample_acc += tc as f64;
        while self.sample_acc >= CYCLES_PER_SAMPLE {
            self.sample_acc -= CYCLES_PER_SAMPLE;
            let (l, r) = self.mix();
            self.samples.push(l);
            self.samples.push(r);
        }
    }

    fn tick_frame_sequencer(&mut self) {
        match self.fs_step {
            0 | 4 => self.clock_length(),
            2 | 6 => {
                self.clock_length();
                self.ch1.step_sweep();
            }
            7 => self.clock_envelope(),
            _ => {}
        }
        self.fs_step = (self.fs_step + 1) & 7;
    }

    fn clock_length(&mut self) {
        self.ch1.sq.step_length();
        self.ch2.step_length();
        self.ch3.step_length();
        self.ch4.step_length();
    }

    fn clock_envelope(&mut self) {
        self.ch1.sq.step_envelope();
        self.ch2.step_envelope();
        self.ch4.step_envelope();
    }

    fn mix(&mut self) -> (f32, f32) {
        let s1 = self.ch1.sq.sample() as f32 / 15.0;
        let s2 = self.ch2.sample() as f32 / 15.0;
        let s3 = self.ch3.sample() as f32 / 15.0;
        let s4 = self.ch4.sample() as f32 / 15.0;

        let mut raw_l = 0.0f32;
        let mut raw_r = 0.0f32;

        // NR51: bits 7-4 = L (SO2), bits 3-0 = R (SO1)
        if self.nr51 & 0x10 != 0 { raw_l += s1; }
        if self.nr51 & 0x20 != 0 { raw_l += s2; }
        if self.nr51 & 0x40 != 0 { raw_l += s3; }
        if self.nr51 & 0x80 != 0 { raw_l += s4; }
        if self.nr51 & 0x01 != 0 { raw_r += s1; }
        if self.nr51 & 0x02 != 0 { raw_r += s2; }
        if self.nr51 & 0x04 != 0 { raw_r += s3; }
        if self.nr51 & 0x08 != 0 { raw_r += s4; }

        let lvol = ((self.nr50 >> 4) & 7) as f32 / 7.0;
        let rvol = (self.nr50 & 7) as f32 / 7.0;

        // 4 channels max per side, scale by master volume
        raw_l = raw_l / 4.0 * lvol;
        raw_r = raw_r / 4.0 * rvol;

        // High-pass filter removes DC offset introduced by inactive channels
        let out_l = raw_l - self.hp_cap_l;
        let out_r = raw_r - self.hp_cap_r;
        self.hp_cap_l = raw_l - out_l * 0.999;
        self.hp_cap_r = raw_r - out_r * 0.999;

        (out_l, out_r)
    }

    pub fn drain_samples(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.samples)
    }

    pub fn read(&self, addr: u16) -> u8 {
        // When power is off, registers read as 0xFF except NR52 and wave RAM
        if !self.power && addr != 0xFF26 && !(0xFF30..=0xFF3F).contains(&addr) {
            return 0xFF;
        }
        match addr {
            // CH1
            0xFF10 => {
                0x80 | ((self.ch1.sweep_period & 7) << 4)
                     | ((self.ch1.sweep_negate as u8) << 3)
                     | (self.ch1.sweep_shift & 7)
            }
            0xFF11 => 0x3F | (self.ch1.sq.duty << 6),
            0xFF12 => {
                (self.ch1.sq.volume_init << 4)
                    | ((self.ch1.sq.env_add as u8) << 3)
                    | (self.ch1.sq.env_period & 7)
            }
            0xFF13 => 0xFF,
            0xFF14 => 0xBF | ((self.ch1.sq.length_enable as u8) << 6),
            // CH2
            0xFF15 => 0xFF,
            0xFF16 => 0x3F | (self.ch2.duty << 6),
            0xFF17 => {
                (self.ch2.volume_init << 4)
                    | ((self.ch2.env_add as u8) << 3)
                    | (self.ch2.env_period & 7)
            }
            0xFF18 => 0xFF,
            0xFF19 => 0xBF | ((self.ch2.length_enable as u8) << 6),
            // CH3
            0xFF1A => 0x7F | ((self.ch3.dac_on as u8) << 7),
            0xFF1B => 0xFF,
            0xFF1C => 0x9F | ((self.ch3.volume_code & 3) << 5),
            0xFF1D => 0xFF,
            0xFF1E => 0xBF | ((self.ch3.length_enable as u8) << 6),
            // CH4
            0xFF1F => 0xFF,
            0xFF20 => 0xFF,
            0xFF21 => {
                (self.ch4.volume_init << 4)
                    | ((self.ch4.env_add as u8) << 3)
                    | (self.ch4.env_period & 7)
            }
            0xFF22 => {
                (self.ch4.clock_shift << 4)
                    | ((self.ch4.width_mode as u8) << 3)
                    | (self.ch4.divisor_code & 7)
            }
            0xFF23 => 0xBF | ((self.ch4.length_enable as u8) << 6),
            // Master control
            0xFF24 => self.nr50,
            0xFF25 => self.nr51,
            0xFF26 => {
                let ch_bits = ((self.ch4.enabled as u8) << 3)
                    | ((self.ch3.enabled as u8) << 2)
                    | ((self.ch2.enabled as u8) << 1)
                    | (self.ch1.sq.enabled as u8);
                0x70 | ((self.power as u8) << 7) | ch_bits
            }
            // Wave RAM
            0xFF30..=0xFF3F => self.ch3.wave_ram[(addr - 0xFF30) as usize],
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        if !self.power && addr != 0xFF26 && !(0xFF30..=0xFF3F).contains(&addr) {
            return;
        }
        match addr {
            // CH1
            0xFF10 => {
                self.ch1.sweep_period = (val >> 4) & 7;
                self.ch1.sweep_negate = val & 0x08 != 0;
                self.ch1.sweep_shift = val & 7;
            }
            0xFF11 => {
                self.ch1.sq.duty = (val >> 6) & 3;
                self.ch1.sq.length_counter = 64 - (val & 0x3F);
            }
            0xFF12 => {
                self.ch1.sq.volume_init = (val >> 4) & 0xF;
                self.ch1.sq.env_add = val & 0x08 != 0;
                self.ch1.sq.env_period = val & 7;
                self.ch1.sq.dac_on = val & 0xF8 != 0;
                if !self.ch1.sq.dac_on {
                    self.ch1.sq.enabled = false;
                }
            }
            0xFF13 => {
                self.ch1.sq.freq = (self.ch1.sq.freq & 0x700) | val as u16;
            }
            0xFF14 => {
                self.ch1.sq.freq = (self.ch1.sq.freq & 0x00FF) | ((val as u16 & 7) << 8);
                self.ch1.sq.length_enable = val & 0x40 != 0;
                if val & 0x80 != 0 {
                    self.ch1.trigger();
                }
            }
            // CH2
            0xFF15 => {}
            0xFF16 => {
                self.ch2.duty = (val >> 6) & 3;
                self.ch2.length_counter = 64 - (val & 0x3F);
            }
            0xFF17 => {
                self.ch2.volume_init = (val >> 4) & 0xF;
                self.ch2.env_add = val & 0x08 != 0;
                self.ch2.env_period = val & 7;
                self.ch2.dac_on = val & 0xF8 != 0;
                if !self.ch2.dac_on {
                    self.ch2.enabled = false;
                }
            }
            0xFF18 => {
                self.ch2.freq = (self.ch2.freq & 0x700) | val as u16;
            }
            0xFF19 => {
                self.ch2.freq = (self.ch2.freq & 0x00FF) | ((val as u16 & 7) << 8);
                self.ch2.length_enable = val & 0x40 != 0;
                if val & 0x80 != 0 {
                    self.ch2.trigger();
                }
            }
            // CH3
            0xFF1A => {
                self.ch3.dac_on = val & 0x80 != 0;
                if !self.ch3.dac_on {
                    self.ch3.enabled = false;
                }
            }
            0xFF1B => {
                self.ch3.length_counter = 256 - val as u16;
            }
            0xFF1C => {
                self.ch3.volume_code = (val >> 5) & 3;
            }
            0xFF1D => {
                self.ch3.freq = (self.ch3.freq & 0x700) | val as u16;
            }
            0xFF1E => {
                self.ch3.freq = (self.ch3.freq & 0x00FF) | ((val as u16 & 7) << 8);
                self.ch3.length_enable = val & 0x40 != 0;
                if val & 0x80 != 0 {
                    self.ch3.trigger();
                }
            }
            // CH4
            0xFF1F => {}
            0xFF20 => {
                self.ch4.length_counter = 64 - (val & 0x3F);
            }
            0xFF21 => {
                self.ch4.volume_init = (val >> 4) & 0xF;
                self.ch4.env_add = val & 0x08 != 0;
                self.ch4.env_period = val & 7;
                self.ch4.dac_on = val & 0xF8 != 0;
                if !self.ch4.dac_on {
                    self.ch4.enabled = false;
                }
            }
            0xFF22 => {
                self.ch4.clock_shift = (val >> 4) & 0xF;
                self.ch4.width_mode = val & 0x08 != 0;
                self.ch4.divisor_code = val & 7;
            }
            0xFF23 => {
                self.ch4.length_enable = val & 0x40 != 0;
                if val & 0x80 != 0 {
                    self.ch4.trigger();
                }
            }
            // Master control
            0xFF24 => self.nr50 = val,
            0xFF25 => self.nr51 = val,
            0xFF26 => {
                let new_power = val & 0x80 != 0;
                if !new_power && self.power {
                    self.power_off();
                }
                self.power = new_power;
            }
            // Wave RAM
            0xFF30..=0xFF3F => {
                self.ch3.wave_ram[(addr - 0xFF30) as usize] = val;
            }
            _ => {}
        }
    }

    fn power_off(&mut self) {
        let wave_ram = self.ch3.wave_ram;
        self.ch1 = Ch1::new();
        self.ch2 = SquareCh::new();
        self.ch3 = Ch3::new();
        self.ch3.wave_ram = wave_ram; // wave RAM survives power-off
        self.ch4 = Ch4::new();
        self.nr50 = 0;
        self.nr51 = 0;
        self.fs_step = 0;
        self.fs_counter = FS_PERIOD;
    }
}
