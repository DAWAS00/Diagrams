//! The coordinator's part of the Class tool (S5): starting, finishing and
//! ending a placement. Moved out of `shell.rs` unchanged.

#[allow(clippy::wildcard_imports)]
use super::*;

impl ShellState {
    /// Starts placing a new class, if the Class tool is active, `diagram` is
    /// the one shown, no dialog or search is open and nothing is being placed.
    pub(super) fn begin_new_class(
        &mut self,
        ctx: &egui::Context,
        diagram: DiagramId,
        at: class_tool::CanvasPoint,
        visible: class_tool::CanvasSize,
    ) {
        if self.active_diagram() != Some(diagram) || self.dialog.is_some() || self.palette.is_some()
        {
            return;
        }
        if self.class_tool.begin(diagram, at, visible).is_none() {
            return;
        }
        self.clear_selection();
        self.drawer = None;
        self.drawer_return = None;
        let name = class_tool::default_class_name(self.document());
        // A fresh field state: no text undo history, the whole default name
        // selected so typing replaces it.
        let field = class_tool::new_class_field();
        let mut edit = egui::text_edit::TextEditState::default();
        edit.cursor
            .set_char_range(Some(egui::text::CCursorRange::select_all_str(&name)));
        edit.store(ctx, field);
        self.drafts.insert(DraftKey::NewClassName, name);
        self.pending_focus = Some(field);
    }

    /// Before any other request, finishes naming with the current draft.
    pub(super) fn preflight(&mut self) -> Finish {
        if self.class_tool.naming().is_none() {
            return Finish::Completed;
        }
        let name = self
            .drafts
            .get(&DraftKey::NewClassName)
            .cloned()
            .unwrap_or_default();
        self.finish_naming(&name)
    }

    /// The one place naming ends by its name: create the class, ask about an
    /// existing name (D2), or block an invalid one.
    pub(super) fn finish_naming(&mut self, name: &str) -> Finish {
        let Some(placement) = self.class_tool.naming() else {
            return Finish::Completed;
        };
        if rules::class_name_error(name).is_some() {
            self.notice = Some(format!(
                "Error: {BLANK_NAME_WARNING} Name the new class, or press Escape to cancel."
            ));
            self.pending_focus = Some(class_tool::new_class_field());
            return Finish::Blocked;
        }
        match class_tool::decide(self.document(), placement.diagram, name) {
            Decision::Create(name) => {
                self.place_new_class(&name);
                Finish::Completed
            }
            Decision::Ask { name, existing } => {
                self.class_tool
                    .set_phase(placement.id, Phase::Asking { existing });
                self.dialog_return = Some(class_tool::new_class_field());
                self.dialog = Some(DialogState::new(DialogKind::DuplicateClassName {
                    placement: placement.id,
                    name,
                    existing,
                }));
                Finish::Asking
            }
        }
    }

    /// Ends the placement by creating a class named `name` where it was
    /// placed, as one undo step, and selects it.
    pub(super) fn place_new_class(&mut self, name: &str) {
        let Some(placement) = self.class_tool.take() else {
            return;
        };
        self.drafts.remove(&DraftKey::NewClassName);
        let before: Vec<ElementId> = self.document().elements.keys().copied().collect();
        let command = Command::CreateClass {
            diagram: placement.diagram,
            name: name.to_owned(),
            bounds: class_tool::bounds(placement.at, class_tool::CLASS_SIZE),
        };
        if self.execute(&command) {
            let created = self
                .document()
                .elements
                .keys()
                .copied()
                .find(|id| !before.contains(id));
            if let Some(id) = created {
                self.select(id);
            }
        }
    }

    /// Ends the placement by showing `element` in its diagram, at its drawn
    /// size and kept inside the canvas, as one undo step, and selects it.
    pub(super) fn show_existing_here(&mut self, element: ElementId) {
        let Some(placement) = self.class_tool.take() else {
            return;
        };
        self.drafts.remove(&DraftKey::NewClassName);
        let lines = self
            .document()
            .elements
            .get(&element)
            .map_or(0, |element| element.members.len());
        let size = canvas_view::footprint(lines);
        let at = class_tool::fit_inside(placement.at, size, placement.visible);
        let command = Command::ShowInDiagram {
            element,
            diagram: placement.diagram,
            bounds: class_tool::bounds(at, size),
        };
        if self.execute(&command) {
            self.select(element);
        }
    }

    /// Ends the placement without creating anything, closing its dialog.
    pub(super) fn cancel_placement(&mut self) {
        if self.class_tool.take().is_none() {
            return;
        }
        self.drafts.remove(&DraftKey::NewClassName);
        if matches!(
            self.dialog.as_ref().map(|dialog| &dialog.kind),
            Some(DialogKind::DuplicateClassName { .. })
        ) {
            self.dialog = None;
            self.dialog_return = None;
        }
        if self.pending_focus == Some(class_tool::new_class_field()) {
            self.pending_focus = None;
        }
    }

    /// Answers the duplicate-name dialog of `placement`.
    pub(super) fn answer_duplicate(
        &mut self,
        placement: PlacementId,
        name: &str,
        existing: Option<ElementId>,
        answer: DialogAnswer,
    ) {
        let asking = self
            .class_tool
            .placement()
            .is_some_and(|live| live.id == placement && matches!(live.phase, Phase::Asking { .. }));
        if !asking {
            return;
        }
        match (answer, existing) {
            (DialogAnswer::Confirm, _) => {
                self.pending_focus = None;
                self.place_new_class(name);
            }
            (DialogAnswer::Alternative, Some(element)) => {
                self.pending_focus = None;
                self.show_existing_here(element);
            }
            // Cancel (or the unavailable choice): back to the name field.
            (DialogAnswer::Cancel | DialogAnswer::Alternative, _) => {
                self.class_tool.set_phase(placement, Phase::Naming);
            }
        }
    }
}
