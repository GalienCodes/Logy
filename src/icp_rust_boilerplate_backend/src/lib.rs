#[macro_use]
extern crate serde;

use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct BlogPost {
    id: u64,
    title: String,
    content: String,
    author: String,
    created_at: u64,
    updated_at: Option<u64>,
    tips_received: u64,
    comments: Vec<u64>,
    view_count: u64,
    tags: Vec<String>,
}

// a trait that must be implemented for a struct that is stored in a stable struct
impl Storable for BlogPost {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// another trait that must be implemented for a struct that is stored in a stable struct
impl BoundedStorable for BlogPost {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}
// Declare the Comment struct
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Comment {
    id: u64,
    blog_post_id: u64,
    author: String,
    content: String,
    created_at: u64,
}

// Implement the Storable trait for Comment
impl Storable for Comment {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// Implement the BoundedStorable trait for Comment
impl BoundedStorable for Comment {
    const MAX_SIZE: u32 = 1024; // Maximum size for the serialized data
    const IS_FIXED_SIZE: bool = false; // Data size is not fixed
}
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );
    static COMMENT_STORAGE: RefCell<StableBTreeMap<u64, Comment, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))) // Using a new MemoryId for comments
        )
    );

    static STORAGE: RefCell<StableBTreeMap<u64, BlogPost, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));
}
#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct CommentPayload {
    blog_post_id: u64,
    author: String,
    content: String,
    
    
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct BlogPostPayload {
    title: String,
    content: String,
    author: String,
    view_count: u64,
    tags: Vec<String>,
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
}

// a helper method to get a blog post by id. used in get_blog_post/update_blog_post
fn _get_blog_post(id: &u64) -> Option<BlogPost> {
    STORAGE.with(|service| service.borrow().get(id))
}

#[ic_cdk::query]
fn get_blog_post(id: u64) -> Result<BlogPost, Error> {
    match _get_blog_post(&id) {
        Some(blog_post) => Ok(blog_post),
        None => Err(Error::NotFound {
            msg: format!("a blog post with id={} not found", id),
        }),
    }
}

#[ic_cdk::update]
fn add_blog_post(blog_post: BlogPostPayload) -> Option<BlogPost> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
    
    let new_blog_post = BlogPost {
        id,
        title: blog_post.title,
        content: blog_post.content,
        author: blog_post.author,
        created_at: time(),
        updated_at: None,
        tips_received: 0,
        comments: Vec::new(), // Initialize an empty Vec for comments
        view_count: 0,       // Initialize view_count to 0
        tags: Vec::new(),    // Initialize an empty Vec for tags
    };

    do_insert_blog_post(&new_blog_post);
    Some(new_blog_post)
}



#[ic_cdk::update]
fn update_blog_post(id: u64, payload: BlogPostPayload) -> Result<BlogPost, Error> {
    match STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut blog_post) => {
            blog_post.content = payload.content;
            blog_post.title = payload.title;
            blog_post.updated_at = Some(time());
            do_insert_blog_post(&blog_post);
            Ok(blog_post)
        }
        None => Err(Error::NotFound {
            msg: format!("couldn't update a blog post with id={}. blog post not found", id),
        }),
    }
}

#[ic_cdk::update]
fn delete_blog_post(id: u64) -> Result<BlogPost, Error> {
    match STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(blog_post) => Ok(blog_post),
        None => Err(Error::NotFound {
            msg: format!("couldn't delete a blog post with id={}. blog post not found", id),
        }),
    }
}
#[ic_cdk::update]
fn add_comment(payload: CommentPayload) -> Result<Comment, Error> {
    // Generate a new ID for the comment
    let id = ID_COUNTER.with(|counter| {
        let current_id = *counter.borrow().get();
        let _ = counter.borrow_mut().set(current_id + 1);
        current_id
    });

    // Create the comment object
    let comment = Comment {
        id,
        blog_post_id: payload.blog_post_id,
        author: payload.author,
        content: payload.content,
        created_at: time(),
    };

    // Insert the comment into COMMENT_STORAGE
    COMMENT_STORAGE.with(|storage| storage.borrow_mut().insert(id, comment.clone()));

    // Retrieve, update, and reinsert the BlogPost
    let mut blog_post_updated = false;
    STORAGE.with(|storage| {
        let mut storage_borrow = storage.borrow_mut();
        if let Some(blog_post) = storage_borrow.get(&payload.blog_post_id) {
            // Create an updated BlogPost
            let mut updated_blog_post = blog_post.clone();
            updated_blog_post.comments.push(id);

            // Reinsert the updated BlogPost
            storage_borrow.insert(updated_blog_post.id, updated_blog_post);
            blog_post_updated = true;
        }
    });

    if blog_post_updated {
        Ok(comment)
    } else {
        Err(Error::NotFound {
            msg: "Blog post not found".to_string(),
        })
    }
}







#[ic_cdk::query]
fn get_comments_for_post(blog_post_id: u64) -> Result<Vec<Comment>, Error> {
    COMMENT_STORAGE.with(|storage| {
        let comments = storage.borrow()
            .iter()
            .filter(|(_, comment)| comment.blog_post_id == blog_post_id)
            .map(|(_, comment)| comment.clone())
            .collect();
        Ok(comments)
    })
}
#[ic_cdk::query]
fn search_blog_posts_by_author(author_name: String) -> Vec<BlogPost> {
    STORAGE.with(|storage| {
        storage.borrow()
               .iter()
               .filter_map(|(_, post)| {
                   if post.author == author_name {
                       Some(post.clone())
                   } else {
                       None
                   }
               })
               .collect()
    })
}

#[ic_cdk::query]
fn get_blog_post_analytics(post_id: u64) -> Result<BlogPostAnalytics, Error> {
    let blog_post = _get_blog_post(&post_id).ok_or(Error::NotFound {
        msg: format!("Blog post with id={} not found", post_id),
    })?;

    let comment_count = COMMENT_STORAGE.with(|storage| {
        storage.borrow()
               .iter()
               .filter(|(_, comment)| comment.blog_post_id == post_id)
               .count()
    });

    Ok(BlogPostAnalytics {
        view_count: blog_post.view_count,
        tips_received: blog_post.tips_received,
        comment_count,
    })
}

#[derive(candid::CandidType, Deserialize)]
struct BlogPostAnalytics {
    view_count: u64,
    tips_received: u64,
    comment_count: usize,
}

#[ic_cdk::update]
fn tag_blog_post(post_id: u64, tags: Vec<String>) -> Result<(), Error> {
    STORAGE.with(|storage| {
        let mut storage_borrow = storage.borrow_mut();
        if let Some(mut post) = storage_borrow.get(&post_id) {
            post.tags = tags;
            storage_borrow.insert(post.id, post.clone());
            Ok(())
        } else {
            Err(Error::NotFound {
                msg: format!("Blog post with id={} not found", post_id),
            })
        }
    })
}

#[ic_cdk::query]
fn search_blog_posts_by_tag(tag: String) -> Vec<BlogPost> {
    STORAGE.with(|storage| {
        storage.borrow()
               .iter()
               .filter_map(|(_, post)| {
                   if post.tags.contains(&tag) {
                       Some(post.clone())
                   } else {
                       None
                   }
               })
               .collect()
    })
}



#[ic_cdk::update]
fn tip_blog_post(id: u64, amount: u64) -> Result<(), Error> {
    match STORAGE.with(|service| {
        let mut storage = service.borrow_mut();
        if let Some(blog_post) = storage.get(&id) {
            // Update the fields of the retrieved blog post
            let mut updated_blog_post = blog_post.clone();
            updated_blog_post.tips_received += amount;
            
            // Replace the old blog post with the updated version
            storage.insert(id, updated_blog_post);
            Ok(())
        } else {
            Err(Error::NotFound {
                msg: format!("couldn't tip a blog post with id={}. blog post not found", id),
            })
        }
    }) {
        Ok(()) => Ok(()),  // Return Ok(()) instead of Ok(result)
        Err(err) => Err(err),
    }
}


// helper method to perform insert.
fn do_insert_blog_post(blog_post: &BlogPost) {
    STORAGE.with(|service| service.borrow_mut().insert(blog_post.id, blog_post.clone()));
}

// need this to generate candid
ic_cdk::export_candid!();
