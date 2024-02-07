use ahash::HashMap;
use indicatif::ProgressIterator;
use serde_derive::{Deserialize, Serialize};

use crate::routing::{edge::DirectedEdge, path::Path, types::VertexId};

#[derive(Serialize, Deserialize)]
pub struct ShortcutReplacer {
    pub shortcuts: HashMap<DirectedEdge, Vec<VertexId>>,
}

impl ShortcutReplacer {
    pub fn new(shortcuts: &HashMap<DirectedEdge, VertexId>) -> Self {
        let shortcuts = shortcuts
            .iter()
            .map(|(shortcut, vertex)| (shortcut.clone(), vec![*vertex]))
            .collect();
        let mut shortcut_replacer = ShortcutReplacer { shortcuts };
        for _ in 0..5 {
            shortcut_replacer.extend_shortcuts();
        }
        shortcut_replacer
    }

    pub fn extend_shortcuts(&mut self) {
        let shortcuts = self.shortcuts.clone();
        shortcuts
            .iter()
            .progress()
            .for_each(|(shortcut, skiped_verticies)| {
                if let Some(skiped_verticies) = self.fun_name(shortcut, skiped_verticies) {
                    self.shortcuts.insert(shortcut.clone(), skiped_verticies);
                }
            });
    }

    fn fun_name(&self, shortcut: &DirectedEdge, skiped_verticies: &Vec<u32>) -> Option<Vec<u32>> {
        let mut vec = vec![shortcut.tail];
        vec.extend(skiped_verticies);
        vec.push(shortcut.head);

        let mut skiped_verticies = Vec::new();
        for window in vec.windows(2) {
            let edge = DirectedEdge {
                tail: window[0],
                head: window[1],
            };
            let verticies = self.shortcuts.get(&edge)?.clone();
            skiped_verticies.extend(verticies);
        }
        Some(skiped_verticies)
    }

    pub fn get_route(&self, path_with_shortcuts: &Path) -> Path {
        let mut path_with_shortcuts = path_with_shortcuts.clone();
        let mut path = Path {
            verticies: Vec::new(),
            cost: path_with_shortcuts.cost,
        };

        while path_with_shortcuts.verticies.len() >= 2 {
            let head = path_with_shortcuts.verticies.pop().unwrap();
            let tail = *path_with_shortcuts.verticies.last().unwrap();
            let edge = DirectedEdge { tail, head };

            if let Some(skiped_verticies) = self.shortcuts.get(&edge) {
                path_with_shortcuts.verticies.extend(skiped_verticies);
                path_with_shortcuts.verticies.push(edge.head);
            } else {
                path.verticies.push(edge.head);
            }
        }

        path.verticies.push(path_with_shortcuts.verticies[0]);
        path.verticies.reverse();

        path
    }
}
