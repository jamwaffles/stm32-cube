use crate::{cube::Cube, patterns::*, transitions::*};

pub struct TransitionState {
    driver: Transition,
    next_pattern: Pattern,
    start: u32,
}

pub struct State {
    current_start: u32,
    pattern: Pattern,
    transition: Option<TransitionState>,
}

impl State {
    pub fn new(pattern: Pattern) -> Self {
        Self {
            pattern,
            transition: None,
            current_start: 0,
        }
    }

    fn next_pattern(&mut self, time: u32, new_pattern: Pattern, transition: Option<Transition>) {
        if let Some(transition) = transition {
            self.transition = Some(TransitionState {
                driver: transition,
                start: time,
                next_pattern: new_pattern,
            });
        } else {
            self.current_start = time;
            self.transition = None;
            self.pattern = new_pattern;
        }
    }

    pub fn drive(&mut self, time: u32, cube: &mut Cube) {
        let pattern_run_time = time - self.current_start;

        if let Some(t) = self.transition.as_mut() {
            let transition_run_time = time - t.start;

            // Next pattern starts at end of transition
            let next_start = if t.driver.next_start_offset() > 0 {
                t.start + transition_run_time
            }
            // Next pattern starts at the same time as the transition
            else {
                t.start
            };

            let next_pattern_run_time = time - next_start;

            if !t.driver.is_complete(transition_run_time) {
                let update_iter = t.next_pattern.update_iter(next_pattern_run_time);

                for (current, next) in cube.frame_mut().iter_mut().zip(update_iter) {
                    let new = t
                        .driver
                        .transition_pixel(transition_run_time, *current, next);

                    *current = new;
                }
            } else {
                self.pattern = t.next_pattern.clone();
                self.current_start = next_start;
                self.transition = None;
            }
        } else {
            cube.fill_iter(self.pattern.update_iter(pattern_run_time));

            match self.pattern {
                Pattern::Rainbow(ref mut pattern) => {
                    if pattern.completed_cycles(pattern_run_time) >= 3 {
                        self.next_pattern(
                            time,
                            Pattern::SlowRain(SlowRain::default()),
                            Some(Transition::CrossFade(CrossFade::default())),
                        );
                    }
                }
                Pattern::SlowRain(ref mut pattern) => {
                    // "cycles" doesn't mean a lot here as drops have different offsets
                    if pattern.completed_cycles(pattern_run_time) == 3 {
                        self.next_pattern(
                            time,
                            Pattern::Slices(Slices::default()),
                            Some(Transition::FadeToBlack(FadeToBlack::default())),
                        );
                    }
                }
                Pattern::Slices(ref mut pattern) => {
                    if pattern.completed_cycles(pattern_run_time) == 2 {
                        self.next_pattern(
                            time,
                            Pattern::ChristmasPuke(ChristmasPuke::default()),
                            Some(Transition::CrossFade(CrossFade::default())),
                        );
                    }
                }
                Pattern::ChristmasPuke(ref mut pattern) => {
                    if pattern.completed_cycles(pattern_run_time) == 3 {
                        self.next_pattern(
                            time,
                            Pattern::Rainbow(Rainbow::default()),
                            Some(Transition::CrossFade(CrossFade::default())),
                        );
                    }
                }
            }
        }
    }
}
