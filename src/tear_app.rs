use std::{process::{Command, Stdio}, io::{Read, BufReader, BufRead, Write}, fs::File};

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
        struct EscapeSequence {

        }

        let mut token = String::new();
        let mut escape_stack = Vec::new();

        enum ParseState {
            None,
            Escape,
        }

        let mut state = ParseState::None;

        for c in out_str.chars() {
            if c.is_control() {
                match state {
                    ParseState::None => {
                        escape_stack.push(EscapeSequence::default());
                        state = ParseState::Escape
                    },
                    ParseState::Escape => panic!("Unexpected escape character in escape sequence."),
                }
            }
            else {
                match state {
                    ParseState::None => token.push(c),
                    ParseState::Escape => {
                        match c {
                            '[' => (), // open the escape sequence
                            // handle digit for color code
                            // m to terminate the sequence
                            _ => {
                                println!("Unhandled character {c} in escape sequence, reverting to treating this is a token");
                                state = ParseState::None;
                                token.push(c);
                            },
                        }
                    },
                }
            } 
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