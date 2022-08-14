use std::{sync::mpsc::{channel, Sender, TryRecvError}, time::Duration, thread::{sleep, spawn}, io::{self, Write}};
use anyhow::{Result};
use colored::Colorize;

#[cfg(test)]
mod tests {
    use std::{time::Duration, thread::sleep};

    use crate::ProgressBar;

    #[test]
    fn test_bar() {
        let bar = ProgressBar::defaults();
        println!("Starting Progress Bar");
        let active = bar.clone().start();
        sleep(Duration::from_secs(5));
        bar.stop(active, true, true, true).unwrap();
    }
}

#[derive(Debug, Clone)]
pub struct ProgressBar(
    String, // Bar text
    usize, // Bar Size
    bool, // Rotating Clock
    u64, // Speed (in mills)
);

#[allow(dead_code)]
impl ProgressBar {
    
    pub fn new(bar_text: String, bar_size: usize, rotating_clock: bool, speed: u64) -> Self {
        
        // let (sender, rec) = channel();

        Self(bar_text, bar_size, rotating_clock, speed)
    }

    pub fn defaults() -> Self {
        Self("#".to_string(), 10, true, 100)
    }

    pub fn stop(self, active_bar: Sender<bool>, fill_bar: bool, errored: bool, colour_bar: bool) -> Result<()> {

        if fill_bar {

            let icon = if errored { "❌" } else { "✔️" };

            let reserved_chars = format!("{} []",icon);

            let taken_length: u16 = reserved_chars.clone().len().try_into()?;

            let termsize::Size {rows: _, cols} = termsize::get().unwrap();

            let avaible_length = cols - taken_length;

            let filler = self.0.repeat(avaible_length.into());

            if colour_bar {
                if errored {
                    print!("\r{} [{}]", icon, filler.red());
                } else {
                    print!("\r{} [{}]", icon, filler.green());
                }
            } else { print!("\r{} [{}]", icon, filler); }

            match io::stdout().flush() {
                Ok(_) => {},
                Err(_) => {},
            }
        }

        active_bar.send(true)?;

        Ok(())
    }

    pub fn start(self) -> Sender<bool> {

        let buffer_size = self.1;

        // Bar Text cannot be an emjoi!
        let buffer = self.0.repeat(buffer_size);
        let mut itr = 0;
    
        let reserved_chars = "🕛 []";
    
        let reserved_chars: u16 = reserved_chars.len().try_into().unwrap();
        
        let clocks = ["🕛","🕐","🕑","🕒","🕓","🕔","🕕","🕖","🕗","🕘","🕙","🕚"];
        let mut clock_face = 0;

        let (sender, rec) = channel();
   
        spawn(move || loop {
    
            let termsize::Size {rows: _, cols} = termsize::get().unwrap();
    
            let cols_usize: usize = cols.clone().into();
            
            let max_size: usize = (cols-reserved_chars).into();
    
            if itr+buffer_size >= max_size-2 {
                itr = 0;
                print!("\r{}"," ".repeat(cols.clone().into()));
            } else { itr += 1; }
    
            if clock_face >= clocks.len()-1 {
                clock_face = 0
            } else { clock_face += 1 }
    
            let empty_space = " ".repeat(itr.clone());
    
            let end = empty_space.len() + buffer.len();
    
            let end = 
            if (end) == cols_usize - 3 { cols_usize - end + 1 } 
            else if (end) == cols_usize - 1 { 0 }
            else if (end) == cols_usize { 0 }
            else if (end) == cols_usize + 1 { 0 } 
            else { (cols_usize - end + 2 ) - reserved_chars as usize};
    
            let end = " ".repeat(end);
    
            let clock = if self.2 { clocks[clock_face] } else { clocks[0] };
    
            print!("\r{} [{}{}{}]",clock,empty_space,buffer,end);
            match io::stdout().flush() {
                Ok(_) => {},
                Err(_) => {},
            }
            sleep(Duration::from_millis(self.3));

            match rec.try_recv() {
                Ok(_) => {break},
                Err(TryRecvError::Disconnected) => {break}
                Err(TryRecvError::Empty) => {},
            }
    
        });

        sender

    }

}