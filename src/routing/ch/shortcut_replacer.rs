use ahash::HashMap;
use serde_derive::{Deserialize, Serialize};

use crate::routing::{edge::DirectedEdge, path::Path, types::VertexId};

#[derive(Serialize, Deserialize)]
pub struct ShortcutReplacer {
    pub shortcuts: HashMap<DirectedEdge, VertexId>,
}

impl ShortcutReplacer {
    pub fn new(shortcuts: &HashMap<DirectedEdge, VertexId>) -> Self {
        ShortcutReplacer {
            shortcuts: shortcuts.clone(),
        }
    }

    pub fn get_route(&self, path_with_shortcuts: &Path) -> Path {
        let mut path_with_shortcuts = path_with_shortcuts.clone();
        let mut path = Path {
            verticies: Vec::new(),
            cost: path_with_shortcuts.cost,
        };

        while path_with_shortcuts.verticies.len() >= 2 {
            let last_num = path_with_shortcuts.verticies.pop().unwrap();
            let second_last_num = *path_with_shortcuts.verticies.last().unwrap();
            let last = DirectedEdge {
                tail: second_last_num,
                head: last_num,
            };
            if let Some(&middle_node) = self.shortcuts.get(&last) {
                path_with_shortcuts
                    .verticies
                    .extend([middle_node, last.head]);
            } else {
                path.verticies.push(last.head);
            }
        }

        path.verticies.push(path_with_shortcuts.verticies[0]);
        path.verticies.reverse();

        path
    }
}
