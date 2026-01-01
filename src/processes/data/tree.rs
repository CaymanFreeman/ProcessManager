use std::collections::HashMap;

struct Node<'a> {
    process: &'a sysinfo::Process,
    children: Vec<Self>,
}

impl<'a> Node<'a> {
    fn new(process: &'a sysinfo::Process) -> Self {
        Self {
            process,
            children: Vec::new(),
        }
    }

    fn flatten(
        &'a self,
        flat_list: &mut Vec<&'a sysinfo::Process>,
        indentations: &mut Vec<usize>,
        depth: usize,
    ) {
        flat_list.push(self.process);
        indentations.push(depth);
        for child in &self.children {
            child.flatten(flat_list, indentations, depth + 1);
        }
    }
}

pub struct ProcessTree<'a> {
    roots: Vec<Node<'a>>,
}

impl<'a> ProcessTree<'a> {
    pub(crate) fn build(processes: &[&'a sysinfo::Process]) -> Self {
        let mut pid_to_node: HashMap<sysinfo::Pid, Node<'a>> = HashMap::new();
        for process in processes {
            pid_to_node.insert(process.pid(), Node::new(process));
        }

        let mut parent_to_children: HashMap<sysinfo::Pid, Vec<sysinfo::Pid>> = HashMap::new();
        let mut root_pids = Vec::new();

        for process in processes {
            match process.parent() {
                Some(parent_pid) => {
                    parent_to_children
                        .entry(parent_pid)
                        .or_default()
                        .push(process.pid());
                }
                None => {
                    root_pids.push(process.pid());
                }
            }
        }

        fn build_subtree<'a>(
            pid: sysinfo::Pid,
            pid_to_node: &mut HashMap<sysinfo::Pid, Node<'a>>,
            parent_child_map: &HashMap<sysinfo::Pid, Vec<sysinfo::Pid>>,
        ) -> Option<Node<'a>> {
            let mut node = pid_to_node.remove(&pid)?;

            if let Some(child_pids) = parent_child_map.get(&pid) {
                for &child_pid in child_pids {
                    if let Some(child_node) =
                        build_subtree(child_pid, pid_to_node, parent_child_map)
                    {
                        node.children.push(child_node);
                    }
                }
            }

            Some(node)
        }

        let mut roots = Vec::new();
        for root_pid in root_pids {
            if let Some(root_node) = build_subtree(root_pid, &mut pid_to_node, &parent_to_children)
            {
                roots.push(root_node);
            }
        }

        Self { roots }
    }

    pub(crate) fn flattened(&'a self) -> (Vec<&'a sysinfo::Process>, Vec<usize>) {
        let mut flat_list = Vec::new();
        let mut indentations = Vec::new();
        for root_node in &self.roots {
            root_node.flatten(&mut flat_list, &mut indentations, 0);
        }
        (flat_list, indentations)
    }
}
