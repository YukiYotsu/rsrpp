use super::*;

#[tokio::test]
async fn test_save_pdf() {
    let url = "https://arxiv.org/pdf/1706.03762";
    let path = save_pdf(url).await.unwrap();
    assert!(Path::new(&path).exists());

    if Path::new(&path).exists() {
        std::fs::remove_file(&path).unwrap();
    }
}

#[tokio::test]
async fn test_pdf2html_url() {
    let url = "https://arxiv.org/pdf/1706.03762";
    let res = pdf2html(url).await;
    let html = res.unwrap();
    assert!(html.html().contains("arXiv:1706.03762"));
}

#[tokio::test]
async fn test_pdf2html_file() {
    let url = "https://arxiv.org/pdf/1706.03762";
    let response = request::get(url).await.unwrap();
    let bytes = response.bytes().await.unwrap();
    let path = "/tmp/test.pdf";
    let mut file = File::create(path).unwrap();
    std::io::copy(&mut bytes.as_ref(), &mut file).unwrap();

    let res = pdf2html("/tmp/test.pdf").await;
    let html = res.unwrap();
    assert!(html.html().contains("arXiv:1706.03762"));

    if Path::new(path).exists() {
        std::fs::remove_file(path).unwrap();
    }
}

#[tokio::test]
async fn test_parse_html() {
    let url = "https://arxiv.org/pdf/1706.03762";
    let res = pdf2html(url).await;
    let html = res.unwrap();

    let pages = parse_html(&html).unwrap();
    assert!(pages.len() > 0);
    let text = pages[0].blocks[0].lines[0].get_text();
    assert_eq!(
        text.trim(),
        "Provided proper attribution is provided, Google hereby grants permission to"
    );

    for page in pages {
        for block in page.blocks {
            let text = block.get_text();
            let attr = block.attr;
            println!("[{:?}]: {}", attr, text.trim());
        }
    }
}

#[test]
fn test_coordinate_is_intercept() {
    let a = Coordinate::from_rect(0.0, 0.0, 10.0, 10.0);
    let b = Coordinate::from_rect(5.0, 5.0, 15.0, 15.0);
    let c = Coordinate::from_rect(15.0, 15.0, 25.0, 25.0);
    let d = Coordinate::from_rect(0.0, 0.0, 5.0, 5.0);
    let e = Coordinate::from_rect(20.0, 5.0, 25.0, 10.0);
    let f = Coordinate::from_rect(5.0, 20.0, 10.0, 25.0);

    assert!(a.is_intercept(&b));
    assert!(!a.is_intercept(&c));
    assert!(a.is_intercept(&d));
    assert!(!a.is_intercept(&e));
    assert!(!a.is_intercept(&f));
    assert!(!b.is_intercept(&c));
    assert!(!b.is_intercept(&d));
    assert!(!b.is_intercept(&e));
    assert!(!b.is_intercept(&f));
}

#[tokio::test]
async fn test_get_font_sizes() {
    let url = "https://arxiv.org/pdf/1706.03762";
    let res = pdf2html(url).await.unwrap();
    let pages = parse_html(&res).unwrap();
    let font_sizes = get_font_sizes(&pages);
    assert!(font_sizes > 0.0);
}
