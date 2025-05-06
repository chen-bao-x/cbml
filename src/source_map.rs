// source_map.rs

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

pub struct SourceMap {
    pub filepath: String,
    pub source: String,
    line_offsets: Vec<usize>, // 每行的起始偏移（字节位置）
}

impl SourceMap {
    pub fn new(filename: &str, source_code: String) -> Self {
        let mut line_offsets = vec![0];
        for (i, ch) in source_code.char_indices() {
            if ch == '\n' {
                line_offsets.push(i + 1);
            }
        }

        Self {
            filepath: filename.to_string(),
            source: source_code,
            line_offsets,
        }
    }

    pub fn lookup(&self, pos: usize) -> (usize, usize, &str) {
        // 返回 (行号, 列号, 该行文本)
        let line_idx = match self.line_offsets.binary_search(&pos) {
            Ok(i) => i,
            Err(i) => i - 1,
        };

        let line_start = self.line_offsets[line_idx];
        let line_end = self.source[line_start..]
            .find('\n')
            .map(|i| line_start + i)
            .unwrap_or(self.source.len());

        let line_text = &self.source[line_start..line_end];
        let col = pos - line_start;

        (line_idx + 1, col + 1, line_text)
    }

    pub fn report_error(&self, span: Span, message: &str) {
        let (line, col, line_text) = self.lookup(span.start);

        println!("error: {}", message);
        println!("  --> {}:{}:{}", self.filepath, line, col);
        println!("    |");
        println!("{:>3} | {}", line, line_text);
        println!("    | {:>width$}^", "", width = col - 1);
    }
}
