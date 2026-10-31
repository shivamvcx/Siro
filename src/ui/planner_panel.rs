use crate::model::planner::{BomItem, GoalItem};
use crate::model::project::Project;
use egui::{Color32, RichText, ScrollArea, Ui};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlannerTab {
    Goals,
    Bom,
    HackClubAndPins,
}

pub struct PlannerPanel {
    pub current_tab: PlannerTab,
    pub new_goal_text: String,
    pub new_bom_name: String,
    pub new_bom_spec: String,
    pub new_bom_qty: u32,
    pub new_bom_cost: f64,
}

impl Default for PlannerPanel {
    fn default() -> Self {
        Self {
            current_tab: PlannerTab::Bom,
            new_goal_text: String::new(),
            new_bom_name: String::new(),
            new_bom_spec: String::new(),
            new_bom_qty: 1,
            new_bom_cost: 1.00,
        }
    }
}

pub enum PlannerAction {
    None,
    SyncBomToCanvas,
    ExportBomCsv,
}

impl PlannerPanel {
    pub fn ui(&mut self, ui: &mut Ui, project: &mut Project, dark_mode: bool) -> PlannerAction {
        let mut action = PlannerAction::None;

        ui.vertical(|ui| {
            // Header tabs
            ui.horizontal(|ui| {
                if ui.selectable_label(self.current_tab == PlannerTab::Bom, "💰 BOM").clicked() {
                    self.current_tab = PlannerTab::Bom;
                }
                if ui.selectable_label(self.current_tab == PlannerTab::Goals, "🎯 Goals").clicked() {
                    self.current_tab = PlannerTab::Goals;
                }
                if ui.selectable_label(self.current_tab == PlannerTab::HackClubAndPins, "🚀 Hack Club & Pins").clicked() {
                    self.current_tab = PlannerTab::HackClubAndPins;
                }
            });

            ui.separator();

            match self.current_tab {
                PlannerTab::Bom => {
                    action = self.render_bom_tab(ui, project, dark_mode);
                }
                PlannerTab::Goals => {
                    self.render_goals_tab(ui, project, dark_mode);
                }
                PlannerTab::HackClubAndPins => {
                    self.render_hackclub_tab(ui, project, dark_mode);
                }
            }
        });

        action
    }

    fn render_bom_tab(&mut self, ui: &mut Ui, project: &mut Project, dark_mode: bool) -> PlannerAction {
        let mut action = PlannerAction::None;

        // Cost summary cards
        let total_cost = project.planner.total_cost();
        let acquired_cost = project.planner.acquired_cost();
        let remaining_cost = project.planner.remaining_cost();

        let budget_color = if dark_mode { Color32::from_rgb(0, 210, 255) } else { Color32::from_rgb(2, 132, 199) };
        let acquired_color = if dark_mode { Color32::from_rgb(40, 200, 80) } else { Color32::from_rgb(22, 130, 60) };
        let remaining_color = if dark_mode { Color32::from_rgb(250, 180, 40) } else { Color32::from_rgb(180, 100, 10) };

        ui.horizontal(|ui| {
            ui.label(RichText::new("Budget:").size(13.0).strong());
            ui.label(
                RichText::new(format!("${:.2}", total_cost))
                    .size(15.0)
                    .strong()
                    .color(budget_color),
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new(format!("Acquired: ${:.2}", acquired_cost))
                    .size(12.0)
                    .color(acquired_color),
            );
            ui.label(
                RichText::new(format!("Remaining: ${:.2}", remaining_cost))
                    .size(12.0)
                    .color(remaining_color),
            );
        });

        ui.add_space(6.0);

        // Action buttons: Sync to Board Table, Export CSV
        ui.horizontal(|ui| {
            if ui.button("⮯ Sync to Canvas Table").clicked() {
                action = PlannerAction::SyncBomToCanvas;
            }
            if ui.button("📤 Export CSV").clicked() {
                action = PlannerAction::ExportBomCsv;
            }
        });

        ui.separator();

        // BOM Items List
        let mut remove_idx: Option<usize> = None;
        let mut any_edited = false;
        ScrollArea::vertical()
            .max_height(320.0)
            .show(ui, |ui| {
                for (idx, item) in project.planner.bom.iter_mut().enumerate() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            let cb = ui.checkbox(&mut item.acquired, "");
                            if cb.changed() {
                                any_edited = true;
                            }

                            let name_edit = ui.text_edit_singleline(&mut item.name);
                            if name_edit.changed() {
                                any_edited = true;
                            }

                            ui.label(format!("${:.2}", item.line_total()));

                            if ui.small_button("×").clicked() {
                                remove_idx = Some(idx);
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Spec:");
                            let spec_edit = ui.text_edit_singleline(&mut item.spec_value);
                            if spec_edit.changed() {
                                any_edited = true;
                            }

                            ui.label("Qty:");
                            let mut qty_str = item.quantity.to_string();
                            if ui.add(egui::TextEdit::singleline(&mut qty_str).desired_width(35.0)).changed() {
                                if let Ok(q) = qty_str.parse::<u32>() {
                                    item.quantity = q;
                                    any_edited = true;
                                }
                            }

                            ui.label("Unit $:");
                            let mut cost_str = format!("{:.2}", item.unit_cost);
                            if ui.add(egui::TextEdit::singleline(&mut cost_str).desired_width(50.0)).changed() {
                                if let Ok(c) = cost_str.parse::<f64>() {
                                    item.unit_cost = c;
                                    any_edited = true;
                                }
                            }
                        });
                    });
                }
            });

        if any_edited {
            project.mark_edited();
        }

        if let Some(idx) = remove_idx {
            if idx < project.planner.bom.len() {
                project.planner.bom.remove(idx);
                project.mark_edited();
            }
        }

        ui.separator();

        // Add new item form
        ui.label(RichText::new("Add Hardware Component:").strong());
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.new_bom_name);
            ui.text_edit_singleline(&mut self.new_bom_spec);
        });
        ui.horizontal(|ui| {
            ui.label("Qty:");
            let mut qty_str = self.new_bom_qty.to_string();
            if ui.add(egui::TextEdit::singleline(&mut qty_str).desired_width(35.0)).changed() {
                if let Ok(q) = qty_str.parse::<u32>() {
                    self.new_bom_qty = q;
                }
            }

            ui.label("Unit $:");
            let mut cost_str = format!("{:.2}", self.new_bom_cost);
            if ui.add(egui::TextEdit::singleline(&mut cost_str).desired_width(50.0)).changed() {
                if let Ok(c) = cost_str.parse::<f64>() {
                    self.new_bom_cost = c;
                }
            }

            if ui.button("+ Add Item").clicked() {
                if !self.new_bom_name.trim().is_empty() {
                    let item = BomItem::new(
                        self.new_bom_name.trim(),
                        self.new_bom_spec.trim(),
                        self.new_bom_qty,
                        self.new_bom_cost,
                    );
                    project.planner.bom.push(item);
                    project.mark_edited();
                    self.new_bom_name.clear();
                    self.new_bom_spec.clear();
                }
            }
        });

        action
    }

    fn render_goals_tab(
        &mut self,
        ui: &mut Ui,
        project: &mut Project,
        _dark_mode: bool,
    ) {
        ui.label(RichText::new("Milestones & Sprint Goals").strong().size(14.0));
        ui.add_space(6.0);

        let mut remove_idx: Option<usize> = None;
        let mut any_edited = false;
        ScrollArea::vertical()
            .max_height(350.0)
            .show(ui, |ui| {
                for (idx, goal) in project.planner.goals.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        let cb = ui.checkbox(&mut goal.completed, "");
                        if cb.changed() {
                            any_edited = true;
                        }

                        let tag = if goal.is_hackclub_milestone { "🚀 " } else { "• " };
                        let mut label_text = RichText::new(format!("{}{}", tag, goal.title));
                        if goal.completed {
                            label_text = label_text.strikethrough().color(Color32::from_rgb(120, 130, 140));
                        }
                        ui.label(label_text);

                        if ui.small_button("×").clicked() {
                            remove_idx = Some(idx);
                        }
                    });
                }
            });

        if any_edited {
            project.mark_edited();
        }

        if let Some(idx) = remove_idx {
            if idx < project.planner.goals.len() {
                project.planner.goals.remove(idx);
                project.mark_edited();
            }
        }

        ui.separator();

        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.new_goal_text);
            if ui.button("+ Add Goal").clicked() {
                if !self.new_goal_text.trim().is_empty() {
                    project.planner.goals.push(GoalItem {
                        id: Uuid::new_v4(),
                        title: self.new_goal_text.trim().to_string(),
                        completed: false,
                        is_hackclub_milestone: false,
                    });
                    project.mark_edited();
                    self.new_goal_text.clear();
                }
            }
        });
    }

    fn render_hackclub_tab(
        &mut self,
        ui: &mut Ui,
        project: &mut Project,
        _dark_mode: bool,
    ) {
        ui.label(RichText::new("Hack Club Grant & Ship Tracker").strong().size(14.0));
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.label("Project Name:");
            let name_edit = ui.text_edit_singleline(&mut project.planner.hackclub.project_name);
            if name_edit.changed() {
                project.mark_edited();
            }
        });

        ui.horizontal(|ui| {
            ui.label("Status:");
            let status_edit = ui.text_edit_singleline(&mut project.planner.hackclub.ship_status);
            if status_edit.changed() {
                project.mark_edited();
            }
        });

        ui.horizontal(|ui| {
            ui.label("Scrapbook URL:");
            let url_edit = ui.text_edit_singleline(&mut project.planner.hackclub.update_url);
            if url_edit.changed() {
                project.mark_edited();
            }
        });

        ui.horizontal(|ui| {
            ui.label("Slack Channel:");
            let slack_edit = ui.text_edit_singleline(&mut project.planner.hackclub.slack_channel);
            if slack_edit.changed() {
                project.mark_edited();
            }
        });

        ui.separator();

        ui.label(RichText::new("Hardware Pinout & Scratchpad Notes:").strong());
        let notes_edit = ui.text_edit_multiline(&mut project.planner.scratchpad_notes);
        if notes_edit.changed() {
            project.mark_edited();
        }
    }
}
