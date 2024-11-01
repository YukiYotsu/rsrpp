use anyhow::{Error, Result};
use rand::Rng;
use reqwest as request;
use scraper::html;
use scraper::Html;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub enum BlockAttr {
    Title,
    Text,
    Else,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Word {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Word {
    pub fn font_size(&self) -> f32 {
        return self.height;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub words: Vec<Word>,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Line {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Line {
        Line {
            words: Vec::new(),
            x: x,
            y: y,
            width: width,
            height: height,
        }
    }
    pub fn add_word(&mut self, text: String, x: f32, y: f32, width: f32, height: f32) {
        self.words.push(Word {
            text: text.trim().to_string(),
            x: x,
            y: y,
            width: width,
            height: height,
        });
    }
    pub fn get_text(&self) -> String {
        let mut words = Vec::new();
        for word in &self.words {
            words.push(word.text.clone());
        }
        return words.join(" ");
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub lines: Vec<Line>,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub attr: BlockAttr,
}

impl Block {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Block {
        Block {
            lines: Vec::new(),
            x: x,
            y: y,
            width: width,
            height: height,
            attr: BlockAttr::Else,
        }
    }
    pub fn add_line(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.lines.push(Line::new(x, y, width, height));
    }
    pub fn get_text(&self) -> String {
        let mut text = String::new();
        for line in &self.lines {
            text.push_str(&line.get_text());
            text.push_str("\n");
        }
        return text;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub blocks: Vec<Block>,
    pub width: f32,
    pub height: f32,
}

impl Page {
    pub fn new(width: f32, height: f32) -> Page {
        Page {
            blocks: Vec::new(),
            width: width,
            height: height,
        }
    }

    pub fn add_block(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.blocks.push(Block::new(x, y, width, height));
    }

    pub fn get_text(&self) -> String {
        let mut text = String::new();
        for block in &self.blocks {
            text.push_str(&block.get_text());
            text.push_str("\n\n");
        }
        return text;
    }

    pub fn top(&self) -> f32 {
        let mut values: Vec<f32> = Vec::new();
        for block in &self.blocks {
            for line in &block.lines {
                values.push(line.y);
            }
        }
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        return values.first().unwrap().clone();
    }

    pub fn bottom(&self) -> f32 {
        let mut values: Vec<f32> = Vec::new();
        for block in &self.blocks {
            for line in &block.lines {
                values.push(line.y + line.height);
            }
        }
        values.sort_by(|a, b| b.partial_cmp(a).unwrap());
        return values.first().unwrap().clone();
    }
    pub fn left(&self) -> f32 {
        let mut values: Vec<f32> = Vec::new();
        for block in &self.blocks {
            for line in &block.lines {
                values.push(line.x);
            }
        }
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        return values.first().unwrap().clone();
    }

    pub fn right(&self) -> f32 {
        let mut values: Vec<f32> = Vec::new();
        for block in &self.blocks {
            for line in &block.lines {
                values.push(line.x + line.width);
            }
        }
        values.sort_by(|a, b| b.partial_cmp(a).unwrap());
        return values.first().unwrap().clone();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Coordinate {
    pub top_left: Point,
    pub top_right: Point,
    pub bottom_left: Point,
    pub bottom_right: Point,
}

impl Coordinate {
    pub fn from_rect(x1: f32, y1: f32, x2: f32, y2: f32) -> Coordinate {
        Coordinate {
            top_left: Point { x: x1, y: y1 },
            top_right: Point { x: x2, y: y1 },
            bottom_left: Point { x: x1, y: y2 },
            bottom_right: Point { x: x2, y: y2 },
        }
    }

    pub fn from_object(x: f32, y: f32, width: f32, height: f32) -> Coordinate {
        Coordinate {
            top_left: Point { x: x, y: y },
            top_right: Point { x: x + width, y: y },
            bottom_left: Point { x: x, y: y + height },
            bottom_right: Point {
                x: x + width,
                y: y + height,
            },
        }
    }

    pub fn width(&self) -> f32 {
        return self.top_right.x - self.top_left.x;
    }

    pub fn height(&self) -> f32 {
        return self.bottom_left.y - self.top_left.y;
    }

    pub fn is_intercept(&self, other: &Coordinate) -> bool {
        if self.top_left.x >= other.bottom_right.x || self.bottom_right.x <= other.top_left.x {
            return false;
        }
        if self.top_left.y >= other.bottom_right.y || self.bottom_right.y <= other.top_left.y {
            return false;
        }
        return true;
    }

    pub fn get_area(&self) -> f32 {
        return self.width() * self.height();
    }

    pub fn intersection(&self, other: &Coordinate) -> Coordinate {
        let x1 = f32::max(self.top_left.x, other.top_left.x);
        let y1 = f32::max(self.top_left.y, other.top_left.y);
        let x2 = f32::min(self.bottom_right.x, other.bottom_right.x);
        let y2 = f32::min(self.bottom_right.y, other.bottom_right.y);
        return Coordinate::from_rect(x1, y1, x2, y2);
    }

    pub fn iou(&self, other: &Coordinate) -> f32 {
        let dx = f32::min(self.bottom_right.x, other.bottom_right.x) - f32::max(self.top_left.x, other.top_left.x);
        let dy = f32::min(self.bottom_right.y, other.bottom_right.y) - f32::max(self.top_left.y, other.top_left.y);

        if dx <= 0.0 || dy <= 0.0 {
            return 0.0;
        } else {
            let area1 = self.width() * self.height();
            let area2 = other.width() * other.height();
            let inter_area = dx * dy;
            return inter_area / (area1 + area2 - inter_area);
        }
    }
}

pub fn get_font_sizes(pages: &Vec<Page>) -> f32 {
    let mut font_sizes: Vec<f32> = Vec::new();
    let text_area = get_text_area(&pages);
    for page in pages {
        for block in &page.blocks {
            for line in &block.lines {
                let line_coord = Coordinate::from_object(line.x, line.y, line.width, line.height);

                let iou = text_area.iou(&line_coord);
                let intersection = text_area.intersection(&line_coord).get_area();
                let line_area = line_coord.get_area();

                if iou > 0.0 && intersection / line_area > 0.9 {
                    font_sizes.push(line.height);
                }
            }
        }
    }
    let normal_font = sci_rs::stats::median(font_sizes.iter()).0;
    return normal_font;
}

pub fn get_text_area(pages: &Vec<Page>) -> Coordinate {
    let mut left_values: Vec<f32> = Vec::new();
    let mut right_values: Vec<f32> = Vec::new();
    let mut top_values: Vec<f32> = Vec::new();
    let mut bottom_values: Vec<f32> = Vec::new();

    for page in pages {
        left_values.push(page.left());
        right_values.push(page.right());
        top_values.push(page.top());
        bottom_values.push(page.bottom());
    }

    let left = sci_rs::stats::median(left_values.iter()).0;
    let right = sci_rs::stats::median(right_values.iter()).0;
    let top = sci_rs::stats::median(top_values.iter()).0;
    let bottom = sci_rs::stats::median(bottom_values.iter()).0;

    return Coordinate {
        top_left: Point { x: left, y: top },
        top_right: Point { x: right, y: top },
        bottom_left: Point { x: left, y: bottom },
        bottom_right: Point { x: right, y: bottom },
    };
}

pub fn get_block_attr(block: &Block, font_size: f32, text_area: &Coordinate) -> BlockAttr {
    let num_lines = block.lines.len();
    let block_font_size = block.lines.iter().map(|line| line.height).sum::<f32>() / num_lines as f32;
    let block_coord = Coordinate::from_object(block.x, block.y, block.width, block.height);

    let iou = text_area.iou(&block_coord);
    let intersection = text_area.intersection(&block_coord).get_area();
    let block_area = block_coord.get_area();
    let is_in_text_area = iou > 0.0 && intersection / block_area > 0.9;

    if is_in_text_area && num_lines == 1 && block_font_size > font_size {
        return BlockAttr::Title;
    } else if is_in_text_area && num_lines > 1 {
        return BlockAttr::Text;
    } else {
        return BlockAttr::Else;
    }
}
async fn save_pdf(path_or_url: &str) -> Result<String> {
    let mut rng = rand::thread_rng();
    let random_value = rng.gen_range(10000..99999);
    let mut save_path = String::new();
    save_path.push_str("/tmp/pdf_");
    save_path.push_str(&random_value.to_string());
    save_path.push_str(".pdf");
    let save_path = save_path.as_str();
    if path_or_url.starts_with("http") {
        let res = request::get(path_or_url).await;
        if let Err(e) = res {
            return Err(Error::msg(format!("Error: {}", e)));
        };

        let bytes = res.unwrap().bytes().await;
        if let Err(e) = bytes {
            return Err(Error::msg(format!("Error: {}", e)));
        };

        let out = File::create(save_path);
        std::io::copy(&mut bytes.unwrap().as_ref(), &mut out.unwrap()).unwrap();

        return Ok(save_path.to_string());
    } else {
        let path = Path::new(path_or_url);
        let res = std::fs::copy(path.as_os_str(), save_path);
        if let Err(e) = res {
            return Err(Error::msg(format!("Error: {}", e)));
        }
    }

    return Ok(save_path.to_string());
}

pub async fn pdf2html(path: &str) -> Result<html::Html> {
    let result = save_pdf(path).await;
    if let Err(e) = result {
        return Err(e);
    }
    let save_path = result.unwrap();

    let html_path = Path::new(&save_path).with_extension("html");

    // parse pdf into html
    let res = Command::new("pdftotext")
        .args(&[
            save_path.to_string(),
            "-nopgbrk".to_string(),
            "-htmlmeta".to_string(),
            "-bbox-layout".to_string(),
            html_path.to_str().unwrap().to_string(),
        ])
        .stdout(Stdio::piped())
        .output();
    if let Err(e) = res {
        return Err(Error::msg(format!("Error: {}", e)));
    }

    let mut html = String::new();
    let mut f = File::open(html_path.clone()).expect("file not found");
    f.read_to_string(&mut html).expect("something went wrong reading the file");
    let html = scraper::Html::parse_document(&html);

    if Path::new(save_path.as_str()).exists() {
        std::fs::remove_file(save_path).unwrap();
    }
    if html_path.exists() {
        std::fs::remove_file(html_path).unwrap();
    }

    return Ok(html);
}

pub fn parse_html(html: &Html) -> Result<Vec<Page>> {
    let mut pages = Vec::new();
    let page_selector = scraper::Selector::parse("page").unwrap();
    let _pages = html.select(&page_selector);
    for page in _pages {
        let page_width = page.value().attr("width").unwrap().parse::<f32>().unwrap();
        let page_height = page.value().attr("height").unwrap().parse::<f32>().unwrap();
        let mut _page = Page::new(page_width, page_height);

        let block_selector = scraper::Selector::parse("block").unwrap();
        let _blocks = page.select(&block_selector);
        for block in _blocks {
            let block_xmin = block.value().attr("xmin").unwrap().parse::<f32>().unwrap();
            let block_ymin = block.value().attr("ymin").unwrap().parse::<f32>().unwrap();
            let block_xmax = block.value().attr("xmax").unwrap().parse::<f32>().unwrap();
            let block_ymax = block.value().attr("ymax").unwrap().parse::<f32>().unwrap();
            let mut _block = Block::new(block_xmin, block_ymin, block_xmax - block_xmin, block_ymax - block_ymin);

            let line_selector = scraper::Selector::parse("line").unwrap();
            let _lines = block.select(&line_selector);
            for line in _lines {
                let line_xmin = line.value().attr("xmin").unwrap().parse::<f32>().unwrap();
                let line_ymin = line.value().attr("ymin").unwrap().parse::<f32>().unwrap();
                let line_xmax = line.value().attr("xmax").unwrap().parse::<f32>().unwrap();
                let line_ymax = line.value().attr("ymax").unwrap().parse::<f32>().unwrap();
                let mut _line = Line::new(line_xmin, line_ymin, line_xmax - line_xmin, line_ymax - line_ymin);

                let word_selector = scraper::Selector::parse("word").unwrap();
                let _words = line.select(&word_selector);
                for word in _words {
                    let word_xmin = word.value().attr("xmin").unwrap().parse::<f32>().unwrap();
                    let word_ymin = word.value().attr("ymin").unwrap().parse::<f32>().unwrap();
                    let word_xmax = word.value().attr("xmax").unwrap().parse::<f32>().unwrap();
                    let word_ymax = word.value().attr("ymax").unwrap().parse::<f32>().unwrap();
                    let text = word.text().collect::<String>();
                    _line.add_word(text, word_xmin, word_ymin, word_xmax - word_xmin, word_ymax - word_ymin);
                }
                _block.lines.push(_line);
            }
            _page.blocks.push(_block);
        }
        pages.push(_page);
    }

    let font_size = get_font_sizes(&pages);
    let text_area = get_text_area(&pages);
    for page in &mut pages {
        for block in &mut page.blocks {
            block.attr = get_block_attr(block, font_size, &text_area);
        }
    }

    return Ok(pages);
}
