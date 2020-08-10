use super::ExecutionState;

#[derive(PartialEq, Default, Clone, Copy)]
struct Timer {
    curr_time: u8,
}

impl Timer {
    fn tick(&mut self) -> Self {
        if self.curr_time > 0 {
            self.curr_time -= 1;
        }
        
        *self
    }
}

#[derive(Default, Clone, Copy)]
pub struct EmulatedTimers {
    sound_timer: Timer,
    delay_timer: Timer,
}

impl EmulatedTimers {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn tick(&mut self) -> Self {
        self.sound_timer.tick();
        self.delay_timer.tick();
        
        if self.sound_timer.curr_time == 1 {
            println!("BEEP!");
        } 
        
        *self
    }
    
    pub fn set_delay_timer(&mut self, value: u8) -> ExecutionState {
        self.delay_timer.curr_time = value;

        ExecutionState::Continue
    }
    
    pub fn set_sound_timer(&mut self, value: u8) -> ExecutionState {
        self.sound_timer.curr_time = value;
        
        ExecutionState::Continue
    }
    
    pub fn get_delay_timer(&self) -> u8 {
        self.delay_timer.curr_time
    }
}


