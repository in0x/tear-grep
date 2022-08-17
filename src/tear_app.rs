use std::{process::{Command, Stdio}, io::{Read, BufReader, BufRead, Write}};

#[derive(Default)]
pub struct App {
    is_open: bool,

    search_text: String,

    dir_text: String,

    output_lines: Vec<String>,
}

impl App {
    pub fn new() -> App {
        Default::default()
    }

    fn parse_and_render_text(&mut self, ui: &mut egui::Ui) {
        let mut rg_out = Command::new("rg")
            .stdout(Stdio::piped())
            .arg("search_res") // emoji test 🍳🍳🍳🦝🦝🦝 !!
            .arg("--pretty")
            .spawn().unwrap()
            .stdout.unwrap();

        let mut out_str = String::new();
        rg_out.read_to_string(&mut out_str);

        #[derive(Default)]
        struct TextSegment {
            text: String,
            color_code: Option<i32>,
            bold: bool
        }
        
        enum ParseState {
            None,
            Escape,
        }

        let mut cur_segment = TextSegment::default();
        let mut completed_segments : Vec<TextSegment> = Vec::new();

        let mut state = ParseState::None;
        let mut escape_code = 0;

        for c in out_str.chars() {
            match state {
                ParseState::None => {
                    if c.is_control() {
                        state = ParseState::Escape
                    }
                    else {
                        cur_segment.text.push(c);
                    }
                }
                // if we find two newlines, break into a new filematch
                ParseState::Escape => {
                    match c {
                        '[' => (), // open the escape sequence
                        'm' =>  {
                            // handle the escape code and close this escape sequence
                            match escape_code {
                                0 => {
                                    if !cur_segment.text.is_empty() {
                                        completed_segments.push(cur_segment);
                                    }
                                    cur_segment = TextSegment::default();
                                },
                                1 => cur_segment.bold = true,
                                _ => cur_segment.color_code = Some(escape_code),
                            }

                            escape_code = 0;
                            state = ParseState::None;
                        },
                        '0'..='9' => {
                            let num = match c.to_digit(10) {
                                Some(d) => d as i32,
                                None => {
                                    println!("Failed to convert digit to its numeric value.");
                                    0
                                }
                            };
                            escape_code *= 10;
                            escape_code += num;
                        },
                        // handle digit for color code
                        // m to terminate the sequence
                        _ => {
                            println!("Unhandled character {c} in escape sequence, reverting to treating this as a token");
                            state = ParseState::None;
                            cur_segment.text.push(c);
                        },
                    }
                },
            }
        }

        // try pushing the last segment, since it may not have been terminated by an escape sequence
        if !cur_segment.text.is_empty() {
            completed_segments.push(cur_segment);
        }

        for seg in &completed_segments {
            print!("Style: ");
            if seg.bold {
                print!("bold ");
            }
            
            seg.color_code.and_then(|val| { print!(" color: {val}"); Some(val) } );

            print!("\n");
            println!("Text: {}", seg.text);
        }

        // out_str.as_bytes().into_iter().for_each(|b| {
        //     let c = *b as char;
        //     print!("{} ", c);
        // });

        panic!();
        // ui.selectable_label(false, out_str);
    }

    pub fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        egui::TopBottomPanel::top("wrap_app_top_bar").show(ctx, |ui| {
            // egui::trace!(ui);
            // self.bar_contents(ui, frame);
        
            // ui.button("steve");

            egui::menu::bar(ui, |ui| {
                ui.menu_button("Settings", |ui| {
                    ui.button("minecraft steve");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let search_res = ui.horizontal(|ui| {
                let res = ui.text_edit_singleline(&mut self.search_text);
                ui.separator();
                ui.label("Search Text");
                res
            });

            ui.horizontal(|ui| {
                let res = ui.text_edit_singleline(&mut self.dir_text);
                ui.separator();
                ui.label("Search Directory");
                res
            });

            self.parse_and_render_text(ui);

            // if search_res.inner.changed() {
            //     let search_dir = if self.dir_text.is_empty() {
            //         ".".to_string()
            //     } else { 
            //         self.dir_text.clone() 
            //     };

            //     let mut rg_out = Command::new("rg")
            //     .stdout(Stdio::piped())
            //     .arg(self.search_text.clone())
            //     .arg("--pretty")
                // .arg("--vimgrep")
                // .current_dir(search_dir)
                // .spawn().unwrap()
                // .stdout.unwrap();
    
                // let reader = BufReader::new(rg_out);
                // self.output_lines = BufRead::lines(reader)
                //     .filter_map(|line| line.ok())
                //     .collect();

                // let mut out_str = String::new();
                // rg_out.read_to_string(&mut out_str);
                // self.output_lines.push(out_str);
                // println!("{}", out_str);

                // self.output_lines = out_str.split("\n\n")
                //     .into_iter()
                //     .map(|slc| slc.to_string())
                //     .collect::<Vec<_>>();
            // }

            // egui::ScrollArea::vertical().show(ui, |ui| {
                // for line in &self.output_lines {
                //     ui.selectable_label(false, line);
                //     ui.separator();
                // }
            // });
        });

        // egui::SidePanel::right("test_right_panel").show(ctx, |ui| {
        //     ui.label("App Log");
        //     ui.separator();
            
        //     ui.label("log goes here");
        // });
    }
}