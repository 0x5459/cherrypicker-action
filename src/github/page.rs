use futures::Future;
use octocrab::Page;

pub trait Pageable {
    type Item;
    type Error;

    type Fut: Future<Output = Result<Page<Self::Item>, Self::Error>>;

    fn list_by_page(&self, page: u32) -> Self::Fut;
}

pub async fn list_all<P: Pageable>(p: P) -> Result<Vec<P::Item>, P::Error> {
    let page = 0;
    let mut v = Vec::new();

    loop {
        let items = p.list_by_page(page).await?;
        if items.next.is_none() {
            break;
        }
        v.extend(items.items);
    }

    Ok(v)
}
