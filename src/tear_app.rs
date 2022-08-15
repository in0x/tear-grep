use std::{process::{Command, Stdio}, io::{Read, BufReader, BufRead}, os::unix::prelude::AsRawFd};

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

            if search_res.inner.changed() {
                let search_dir = if self.dir_text.is_empty() {
                    "./".to_string()
                } else { 
                    self.dir_text.clone() 
                };

                let mut rg_out = Command::new("rg")
                .stdout(Stdio::piped())
                .args([self.search_text.clone()])
                .current_dir(search_dir)
                .spawn().unwrap()
                .stdout.unwrap();
    
                let reader = BufReader::new(rg_out);
        
                self.output_lines = BufRead::lines(reader)
                    .filter_map(|line| line.ok())
                    .collect();
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                for line in &self.output_lines {
                    ui.label(line);
                    ui.separator();
                }
            });
        });

        egui::SidePanel::right("test_right_panel").show(ctx, |ui| {
            ui.label("App Log");
            ui.separator();
            
            ui.label("log goes here");
        });

        // match rg_process.wait() {
        //     Ok(_) => (),
        //     Err(_) => println!("error waiting for rg"),
        // }

        // rg_process.

        // rg_process.stdout.unwrap().read_to_string(buf)

        // .kill()
        // .output()
        // .expect("failed to run ripgrep"); // handle more gracefully, check it works when first launching


        // egui::Window::new("tear_app")
        //     .open(&mut self.is_open)
        //     .show(ctx, |ui| {
                
        //     });
    }
}