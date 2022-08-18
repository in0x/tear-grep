use std::{process::{Command, Stdio}, io::{Read}, path::Path};

// https://github.com/jan-warchol/selenized/blob/master/the-values.md
const TG_RED :    egui::Color32 = egui::Color32::from_rgb(250,  87,  80);
const TG_GREEN :  egui::Color32 = egui::Color32::from_rgb(117, 185,  56);
const TG_MAGENTA: egui::Color32 = egui::Color32::from_rgb(242, 117, 190);

#[derive(Default)]
pub struct App {
    is_open: bool,

    has_rg_installed: Option<bool>,

    search_text: String,

    dir_text: String,

    result_layout: Vec<egui::text::LayoutJob>,
}

fn parse_and_layout_text(text_to_parse: &str) -> Vec<egui::text::LayoutJob> {
    #[derive(Default)]
    struct TextSegment {
        text: String,
        color_code: Option<i32>,
        bold: bool,
        terminates_job: bool,
    }
    
    enum ParseState {
        None,
        Escape,
        Newline,
    }

    let mut cur_segment = TextSegment::default();
    let mut completed_segments : Vec<TextSegment> = Vec::new();

    let mut state = ParseState::None;
    let mut escape_code = 0;

    for c in text_to_parse.chars() {
        match state {
            ParseState::None => {
                match c {
                    '\u{1b}' => state = ParseState::Escape,
                    _ => {
                        if c == '\n' {
                            state = ParseState::Newline;
                        }
                        
                        cur_segment.text.push(c) 
                    },
                }
            }
            ParseState::Newline => {
                match c {
                    '\u{1b}' => state = ParseState::Escape,
                    '\n' => {
                        if !cur_segment.text.is_empty() {
                            completed_segments.push(cur_segment);
                        }
                        cur_segment = TextSegment::default();

                        completed_segments.push(TextSegment {
                            terminates_job: true,
                            ..Default::default()
                        });

                        state = ParseState::None;
                    }
                    _ => {
                        state = ParseState::None;
                        cur_segment.text.push(c) 
                    }
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

    // fn get_color(escape_code: Option<i32>) -> egui::Color32 {
    //     match escape_code {
    //         Some(31) => egui::Color32::from_rgb(243, 129, 129), // Lines
    //         Some(32) => egui::Color32::from_rgb(234, 255, 208), // Matches
    //         Some(35) => egui::Color32::from_rgb(149, 225, 211), // Files Names
    //         None => egui::Color32::GRAY,                        // Text
    //         _ => egui::Color32::DEBUG_COLOR,                    // Unknown colors
    //     }
    // }

    fn get_color(escape_code: Option<i32>) -> egui::Color32 {
        match escape_code {
            Some(31) => TG_RED,              // Matches
            Some(32) => TG_GREEN,            // Lines
            Some(35) => TG_MAGENTA,          // Files Names
            None => egui::Color32::GRAY,     // Text
            _ => egui::Color32::DEBUG_COLOR, // Unknown colors
        }
    }

    let mut all_layouts = Vec::new();
    let mut layout_job = egui::text::LayoutJob::default();
    for seg in &completed_segments {
        if seg.terminates_job {
            all_layouts.push(layout_job);
            layout_job = Default::default();

            continue;
        }

        layout_job.append(
            &seg.text, 
            0.0, 
            egui::text::TextFormat {
                color: get_color(seg.color_code),
                background: egui::Color32::TRANSPARENT,
                ..Default::default()
            }
        );
    }

    if !layout_job.is_empty() {
        all_layouts.push(layout_job);
    }

    all_layouts
}

fn detect_rg_install() -> bool {
    let mut spawn_result = Command::new("rg")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    match spawn_result {
        Ok(_) => true,
        Err(_) => false
    }
}

impl App {
    pub fn new() -> App {
        Default::default()
    }

    pub fn render_help(ctx: &egui::Context, frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 3.0;
                ui.label("You are missing some required software! tear-grep requires ripgrep to be installed to work. Please follow the instructions");
                ui.hyperlink_to("on ripgrep's github to", "https://github.com/BurntSushi/ripgrep#installation/");
                ui.label("install the software. Make sure the 'rg' command works, then restart this program.");
            });
        });
    }

    pub fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        egui::TopBottomPanel::top("wrap_app_top_bar").show(ctx, |ui| {
            // egui::trace!(ui);
            // self.bar_contents(ui, frame);
            
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Settings", |ui| {
                    ui.button("minecraft steve");
                });
            });
        });

        match self.has_rg_installed {
            None => {
                self.has_rg_installed = Some(detect_rg_install());
                return;
            },
            Some(false) => { // user doesnt have ripgrep, installed, disable controls and show them help instead
                App::render_help(ctx, frame);
                return;
            }
            Some(true) => (), // all good, user has what we need to run with.
        }

        fn is_dir_valid(dir: &str) -> bool {
            Path::is_dir(Path::new(dir)) || dir.is_empty()
        }

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

                if is_dir_valid(&self.dir_text) {
                    ui.label("Search Directory");
                }
                else {
                    let mut layout_job = egui::text::LayoutJob::default();
                    layout_job.append(
                        "Search Directory (Invalid)", 
                        0.0, 
                        egui::text::TextFormat {
                            color: TG_RED,
                            background: egui::Color32::TRANSPARENT,
                            italics: true,
                            ..Default::default()
                        }
                    );

                    ui.label(layout_job);
                }

                res
            });

            // emoji test 🍳🍳🍳🦝🦝🦝 !!

            let mut run_search = search_res.inner.changed();
            run_search &= !self.search_text.is_empty(); 
            run_search &= is_dir_valid(&self.dir_text);

            if run_search {
                let search_dir = if !self.dir_text.is_empty() {
                    &self.dir_text
                } 
                else {
                    "."
                };

                let mut rg_proc = Command::new("rg")
                    .stdout(Stdio::piped())
                    .current_dir(search_dir)
                    .arg(self.search_text.clone())
                    .arg("--pretty")
                    .arg("--threads").arg("8")
                    .spawn().unwrap();

                rg_proc.try_wait();

                let mut rg_out = rg_proc.stdout.unwrap();
                
                let mut result_string = String::new();
                rg_out.read_to_string(&mut result_string);

                self.result_layout = parse_and_layout_text(&result_string);
            }
            else if self.search_text.is_empty() {
                self.result_layout.clear();
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                for layout in &self.result_layout {
                    ui.selectable_label(false, layout.clone());    
                }
            });
        });

        // egui::SidePanel::right("test_right_panel").show(ctx, |ui| {
        //     ui.label("App Log");
        //     ui.separator();
            
        //     ui.label("log goes here");
        // });
    }
}