use similar::{ChangeTag, TextDiff};

#[derive(Clone, Debug)]
pub struct DiffLine {
    pub tag: char, // ' ', '+', '-'
    pub old_line: Option<usize>,
    pub new_line: Option<usize>,
    pub content: String,
}

#[derive(Clone, Debug)]
pub struct DiffHunk {
    pub file_path: String,
    pub added_count: usize,
    pub removed_count: usize,
    pub lines: Vec<DiffLine>,
}

pub fn compute_diff(file_path: &str, old_content: &str, new_content: &str) -> DiffHunk {
    let diff = TextDiff::from_lines(old_content, new_content);
    let mut lines = Vec::new();
    let mut added_count = 0;
    let mut removed_count = 0;

    for change in diff.iter_all_changes() {
        let tag = match change.tag() {
            ChangeTag::Delete => {
                removed_count += 1;
                '-'
            }
            ChangeTag::Insert => {
                added_count += 1;
                '+'
            }
            ChangeTag::Equal => ' ',
        };
        lines.push(DiffLine {
            tag,
            old_line: change.old_index().map(|i| i + 1),
            new_line: change.new_index().map(|i| i + 1),
            content: change.value().trim_end_matches(['\r', '\n']).to_string(),
        });
    }

    DiffHunk {
        file_path: file_path.to_string(),
        added_count,
        removed_count,
        lines,
    }
}
