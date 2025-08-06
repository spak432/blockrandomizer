use eframe::egui;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::{HashMap, VecDeque};
use std::fs::{OpenOptions};
use std::io::{Write};
use rust_xlsxwriter::{Workbook, Format, XlsxError};

#[derive(Clone)]
struct BlockRandomizer {
    block_size: usize,
    groups: Vec<String>,
    queue: VecDeque<String>,
}

impl BlockRandomizer {
    fn new(block_size: usize, groups: Vec<String>) -> Self {
        let mut r = BlockRandomizer {
            block_size,
            groups,
            queue: VecDeque::new(),
        };
        r.generate_block();
        r
    }

    fn generate_block(&mut self) {
        let mut rng = thread_rng();
        let mut block: Vec<String> = self.groups
            .iter()
            .cloned()
            .cycle()
            .take(self.block_size)
            .collect();
        block.shuffle(&mut rng);
        self.queue.extend(block);
    }

    fn generate_block_with_priority(&mut self, priority: &str) {
        let mut rng = thread_rng();
        let mut block: Vec<String> = self.groups
            .iter()
            .cloned()
            .cycle()
            .take(self.block_size)
            .collect();
        // 부족한 군이 우선 오도록 block 재정렬
        block.sort_by(|a, b| if a == priority { std::cmp::Ordering::Less } else { a.cmp(b) });
        block.shuffle(&mut rng);
        self.queue.extend(block);
    }

    fn assign(&mut self) -> String {
        if self.queue.is_empty() {
            self.generate_block();
        }
        self.queue.pop_front().unwrap()
    }
}

struct App {
    randomizers: HashMap<(String, String), BlockRandomizer>,
    subject_id: String,
    gender: String,
    age: u32,
    log: Vec<(String, String, String)>, // (ID, Strata, Group)
    block_size: usize,
    counts_cache: HashMap<(String, String), (usize, usize)>,
    total_cache: (usize, usize),
}

impl App {
    fn load_csv(&mut self) {
        if std::path::Path::new("assignments.csv").exists() {
            let mut rdr = csv::Reader::from_path("assignments.csv").unwrap();
            for result in rdr.records() {
                let record = result.unwrap();
                let id = record[0].to_string();
                let strata = record[1].to_string();
                let group = record[2].to_string();
                self.log.push((id, strata, group));
            }
        }
    }

    fn save_to_csv(&self, record: &(String, String, String)) {
        let file_exists = std::path::Path::new("assignments.csv").exists();

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("assignments.csv")
            .unwrap();

        if !file_exists {
            writeln!(file, "Subject ID,Strata,Group").unwrap();
        }
        writeln!(file, "{},{},{}", record.0, record.1, record.2).unwrap();
    }

    fn save_to_excel(&self) -> Result<(), XlsxError> {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        let header_format = Format::new().set_bold();

        worksheet.write(0, 0, "Subject ID")?;
        worksheet.write(0, 1, "Strata")?;
        worksheet.write(0, 2, "Group")?;
        worksheet.set_row_format(0, &header_format)?;

        for (i, (id, strata, group)) in self.log.iter().enumerate() {
            worksheet.write((i + 1) as u32, 0, id)?;
            worksheet.write((i + 1) as u32, 1, strata)?;
            worksheet.write((i + 1) as u32, 2, group)?;
        }

        workbook.save("assignments.xlsx")?;
        Ok(())
    }

    fn get_counts(&self) -> HashMap<(String, String), (usize, usize)> {
        let mut counts: HashMap<(String, String), (usize, usize)> = HashMap::new();
        for (_, strata, group) in &self.log {
            let parts: Vec<&str> = strata.split('/').map(|s| s.trim()).collect();
            let gender = parts[0].to_string();
            let age_group = parts[1].to_string();
            let entry = counts.entry((gender, age_group)).or_insert((0, 0));
            if group == "A" {
                entry.0 += 1;
            } else {
                entry.1 += 1;
            }
        }
        counts
    }
    
    fn total_counts(&self) -> (usize, usize) {
        let total_a = self.log.iter().filter(|(_, _, g)| g == "A").count();
        let total_b = self.log.iter().filter(|(_, _, g)| g == "B").count();
        (total_a, total_b)
    }

    fn recalc_counts(&mut self) {
        self.counts_cache = self.get_counts();
        self.total_cache = self.total_counts();
    }

    fn assign_next(&mut self) -> String {
        let age_strata = if self.age < 55 { "<55" } else { "≥55" };
        let strata_key = (self.gender.clone(), age_strata.to_string());

        let (total_a, total_b) = self.total_counts();
        let counts = self.get_counts();
        let strata_counts = counts.get(&strata_key).unwrap_or(&(0, 0));

        let prefer_group = if (total_a as i32 - total_b as i32).abs() >= (self.block_size / 2) as i32 {
            if total_a > total_b { "B" } else { "A" }
        } else if (strata_counts.0 as i32 - strata_counts.1 as i32).abs() >= (self.block_size / 2) as i32 {
            if strata_counts.0 > strata_counts.1 { "B" } else { "A" }
        } else {
            ""
        };

        let r = self.randomizers.get_mut(&strata_key).unwrap();
        let group = if prefer_group != "" {
            r.generate_block_with_priority(prefer_group);
            r.assign()
        } else {
            r.assign()
        };
        group
    }
}

impl Default for App {
    fn default() -> Self {
        let mut randomizers = HashMap::new();
        let genders = vec!["Male", "Female"];
        let ages = vec!["<55", "≥55"];
        for g in &genders {
            for a in &ages {
                randomizers.insert(
                    (g.to_string(), a.to_string()),
                    BlockRandomizer::new(4, vec!["A".to_string(), "B".to_string()]),
                );
            }
        }
        let mut app = Self {
            randomizers,
            subject_id: "".to_string(),
            gender: "Male".to_string(),
            age: 50,
            log: vec![],
            block_size: 4,
            counts_cache: HashMap::new(),
            total_cache: (0, 0),
        };
        app.load_csv(); // CSV 자동 로드
        app
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Stratified Block Randomization (A/B 1:1 Priority)");

            ui.horizontal(|ui| {
                ui.label("Subject ID:");
                ui.text_edit_singleline(&mut self.subject_id);
            });

            ui.horizontal(|ui| {
                ui.label("Gender:");
                egui::ComboBox::from_label("")
                    .selected_text(&self.gender)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.gender, "Male".to_string(), "Male");
                        ui.selectable_value(&mut self.gender, "Female".to_string(), "Female");
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Age:");
                ui.add(egui::DragValue::new(&mut self.age));
            });

            if ui.button("Assign Group").clicked() {
                let age_strata = if self.age < 55 { "<55" } else { "≥55" };
                let group = self.assign_next();
                let record = (
                    self.subject_id.clone(),
                    format!("{} / {}", self.gender, age_strata),
                    group.clone(),
                );
                self.log.push(record.clone());
                self.save_to_csv(&record);
                let _ = self.save_to_excel();
                self.subject_id.clear();

                self.recalc_counts();
            }

            ui.separator();
            ui.heading("Current Balance");


            for ((g, a), (ca, cb)) in &self.counts_cache {
                ui.label(format!("{} / {}: A={}  B={}  Δ={}", g, a, ca, cb, (*ca as i32 - *cb as i32).abs()));
            }

            ui.separator();
            let (total_a, total_b) = self.total_cache;
            ui.label(format!("Total: A={}  B={}  Δ={}", total_a, total_b, (total_a as i32 - total_b as i32).abs()));

            ui.separator();
            ui.heading("Assignment Log");
            for (id, strata, group) in &self.log {
                ui.label(format!("{} → [{}] {}", id, strata, group));
            }
            // let counts = self.get_counts();
            // for ((g, a), (ca, cb)) in &counts {
            //     ui.label(format!("{} / {}: A={}  B={}  Δ={}", g, a, ca, cb, (*ca as i32 - *cb as i32).abs()));
            // }

            // let (total_a, total_b) = self.total_counts();
            // ui.separator();
            // ui.label(format!("Total: A={}  B={}  Δ={}", total_a, total_b, (total_a as i32 - total_b as i32).abs()));

            // ui.separator();
            // ui.heading("Assignment Log");
            // for (id, strata, group) in &self.log {
            //     ui.label(format!("{} → [{}] {}", id, strata, group));
            // }
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Randomization App",
        options,
        Box::new(|_cc| Ok(Box::new(App::default()))),
    )
}