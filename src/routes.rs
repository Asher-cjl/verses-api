use actix_web::{get, post, web, HttpResponse};
use rand::{rng, Rng};
use serde_json::json;
use sqlx::{Postgres, QueryBuilder};

use crate::error::AppError;
use crate::models::*;
use crate::AppData;
use crate::constants;

#[allow(unused_assignments)]
pub async fn query_verses(qp: web::Query<VerseFilter>, app_data: web::Data<AppData>) -> Vec<Verse> {
    let mut is_first = true;
    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"SELECT translation, book, abbreviation, book_name, chapter, verse_number, verse FROM fulltable"#,
    );

    if let Some(x) = &qp.abbreviation {
        query_builder.push(" WHERE abbreviation=");
        is_first = false;
        query_builder.push_bind(x.to_uppercase());
    }
    if let Some(x) = &qp.ab {
        if is_first {
            query_builder.push(" WHERE abbreviation=");
            is_first = false;
        } else {
            query_builder.push(" AND abbreviation=");
        }
        query_builder.push_bind(x.to_uppercase());
    }
    if let Some(x) = &qp.book {
        if is_first {
            query_builder.push(" WHERE book=");
            is_first = false;
        } else {
            query_builder.push(" AND book=");
        }
        query_builder.push_bind(x);
    }
    if let Some(x) = &qp.b {
        if is_first {
            query_builder.push(" WHERE book=");
            is_first = false;
        } else {
            query_builder.push(" AND book=");
        }
        query_builder.push_bind(x);
    }
    if let Some(x) = &qp.ch {
        query_builder.push(" AND chapter=");
        query_builder.push_bind(x);
    }
    if let Some(x) = &qp.chapter {
        query_builder.push(" AND chapter=");
        query_builder.push_bind(x);
    }
    if let Some(x) = &qp.sch {
        query_builder.push(" AND chapter>=");
        query_builder.push_bind(x);
    }
    if let Some(x) = &qp.startchapter {
        query_builder.push(" AND chapter>=");
        query_builder.push_bind(x);
    }
    if let Some(x) = &qp.ech {
        query_builder.push(" AND chapter<=");
        query_builder.push_bind(x);
    }
    if let Some(x) = &qp.endchapter {
        query_builder.push(" AND chapter<=");
        query_builder.push_bind(x);
    }
    if let Some(x) = &qp.v {
        query_builder.push(" AND verse_number=");
        query_builder.push_bind(x);
    }
    if let Some(x) = &qp.verse {
        query_builder.push(" AND verse_number=");
        query_builder.push_bind(x);
    }
    if let Some(x) = &qp.sv {
        query_builder.push(" AND verse_number>=");
        query_builder.push_bind(x);
    }
    if let Some(x) = &qp.startverse {
        query_builder.push(" AND verse_number>=");
        query_builder.push_bind(x);
    }
    if let Some(x) = &qp.ev {
        query_builder.push(" AND verse_number<=");
        query_builder.push_bind(x);
    }
    if let Some(x) = &qp.endverse {
        query_builder.push(" AND verse_number<=");
        query_builder.push_bind(x);
    }
    if let Some(x) = &qp.tr {
        query_builder.push(" AND translation=");
        query_builder.push_bind(x.to_uppercase());
    }
    if let Some(x) = &qp.translation {
        query_builder.push(" AND translation=");
        query_builder.push_bind(x.to_uppercase());
    }
    query_builder.push(" ORDER BY id");
    let query = query_builder.build_query_as::<Verse>();
    let verses = query.fetch_all(&app_data.pool).await.unwrap();
    return verses;
}

/// Hello Message
#[utoipa::path(
    get,
    tag = "Hello",
    path = "/",
    responses(
        (status = 200, description = "API healthy", body = Hello)
    )
)]
#[get("/")]
pub async fn home() -> HttpResponse {
    let hello = Hello::default();
    return HttpResponse::Ok().json(hello);
}

/// Get a list of available translations
#[utoipa::path(
    get,
    tag = "Info",
    path = "/translations",
    responses(
        (status = 200, description = "List of bible translations available", body = TranslationInfo)
    )
)]
#[get("/translations")]
pub async fn get_translations(app_data: web::Data<AppData>) -> HttpResponse {
    let q = sqlx::query_as!(TranslationInfo, r#"
        SELECT
            t.name AS name,
            l.lname AS language,
            t.full_name,
            t.seo_name,
            t.regional_name,
            ottn.otname AS ot_name,
            nttn.ntname AS nt_name,
            t.year,
            t.license,
            t.description
        FROM "Translation" t
        JOIN (
            SELECT
                id,
                name AS lname
            FROM "Language"
        ) l
            ON l.id = t.language_id
        JOIN (
            SELECT
                translation_id,
                name AS otname
            FROM "TestamentName"
            WHERE testament = 'OldTestament'
        ) ottn
            ON ottn.translation_id = t.id
        JOIN (
            SELECT
                translation_id,
                name AS ntname
            FROM "TestamentName"
            WHERE testament = 'NewTestament'
        ) nttn
            ON nttn.translation_id = t.id
        ORDER BY
            t.id;
    "#).fetch_all(&app_data.pool).await.unwrap();
    return HttpResponse::Ok().json(q);
}

/// Get a list of Bible books
#[utoipa::path(
    get,
    tag = "Info",
    path = "/books",
    params (TranslationSelector),
    responses(
        (status = 200, description = "List of bible books")
    )
)]
#[get("/books")]
pub async fn get_books(
    qp: web::Query<TranslationSelector>,
    app_data: web::Data<AppData>,
) -> HttpResponse {
    let mut ot = Vec::new();
    let mut nt = Vec::new();
    let mut translation_name = String::new();
    let q;
    if qp.translation.is_some() {
        translation_name = qp.translation.clone().unwrap();
    } else if qp.tr.is_some() {
        translation_name = qp.tr.clone().unwrap();
    }
    if !translation_name.is_empty() {
        q = sqlx::query_as!(BookName, r#"SELECT name from "TranslationBookName" where translation_id=(select id from "Translation" where name=$1) order by id"#, translation_name.to_uppercase()).fetch_all(&app_data.pool).await.unwrap();
    } else {
        q = sqlx::query_as!(BookName, r#"SELECT name FROM "Book" order by id"#)
            .fetch_all(&app_data.pool)
            .await
            .unwrap();
    }
    if q.len() == 66 {
        for i in 0..39 {
            ot.push(q[i].name.clone());
        }
        for i in 39..66 {
            nt.push(q[i].name.clone());
        }
    } else {
        return HttpResponse::BadRequest().json(json!({
            "message": "Not all books were fetched, check if the translation name is correct"
        }));
    }
    return HttpResponse::Ok().json(json!({
        "Old Testament": ot,
        "New Testament": nt,
    }));
}

/// Get a list of abbreviations of books
#[utoipa::path(
    get,
    tag = "Info",
    path = "/abbreviations",
    responses(
        (status = 200, description = "Get a list of abbreviations supported"),
    ),
)]
#[get("/abbreviations")]
pub async fn get_abbreviations(app_data: web::Data<AppData>) -> HttpResponse {
    let q = sqlx::query!(r#"SELECT abbreviation from "Book" order by id"#)
        .fetch_all(&app_data.pool)
        .await
        .unwrap();
    let mut v = Vec::new();
    for i in q {
        v.push(i.abbreviation.clone());
    }
    return HttpResponse::Ok().json(v);
}

/// Get a list of verses by filtering with VerseFilter
#[utoipa::path(
    get,
    tag = "Verse",
    path = "/verses",
    params (VerseFilter),
    responses(
        (status = 200, description = "Get verses based on query parameters", body = Verse),
        (status = 400, description = "Either of book or abbreviation parameters is required")
    ),
)]
#[get("/verses")]
pub async fn get_verses(app_data: web::Data<AppData>, qp: web::Query<VerseFilter>) -> HttpResponse {
    if qp.book.is_none() && qp.b.is_none() && qp.abbreviation.is_none() && qp.ab.is_none() {
        return HttpResponse::BadRequest().json(json!({
            "message": "Either one of book or abbreviation parameters is required"
        }));
    }
    let query = query_verses(qp, app_data).await;
    return HttpResponse::Ok().json(query);
}

/// Get a random verse (not filtered to get good verses)
#[utoipa::path(
    get,
    tag = "Verse",
    path = "/verses/random",
    params (TranslationSelector),
    responses(
        (status = 200, description = "Get a random verse (not filtered for good verses, beware)", body = Verse),
    ),
)]
#[get("/verses/random")]
pub async fn get_random_verse(
    app_data: web::Data<AppData>,
    parameters: web::Query<TranslationSelector>,
) -> HttpResponse {
    let r: i32 = rng().random_range(1..31102);
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
        SELECT
            t.name AS translation,
            b.name AS book,
            b.abbreviation AS abbreviation,
            bb.name AS book_name,
            c.chapter_number AS chapter,
            v.verse_number AS verse_number,
            vv.verse AS verse
        FROM "VerseText" vv
        JOIN "Translation" t
            ON t.id = vv.translation_id
        JOIN "Verse" v
            ON v.id = vv.verse_id
        JOIN "Chapter" c
            ON c.id = v.chapter_id
        JOIN "Book" b
            ON b.id = c.book_id
        JOIN "TranslationBookName" bb
            ON bb.book_id = b.id
        AND vv.translation_id = bb.translation_id
        WHERE
            vv.verse_id=
        "#
    );
    qb.push_bind(r);
    if parameters.translation.is_some() {
        let tr = parameters.translation.clone().unwrap().to_uppercase();
        qb.push(" and t.name=");
        qb.push_bind(tr);
    } else if parameters.tr.is_some() {
        let tr = parameters.tr.clone().unwrap().to_uppercase();
        qb.push(" and t.name=");
        qb.push_bind(tr);
    }
    let query = qb.build_query_as::<Verse>();
    let verses = query.fetch_all(&app_data.pool).await.unwrap();
    return HttpResponse::Ok().json(verses);
}

///Get the number of chapters in all books
///
///This is hardcoded and the endpoint does not use the database
#[utoipa::path(
    get,
    tag = "Info",
    path = "/chaptercount",
    responses(
        (status = 200, description = "Number of chapters in all books", body = BooksChapterCount)
        )
    )
]
#[get("/chaptercount")]
pub async fn get_chaptercount() -> HttpResponse {
    let ans = BooksChapterCount::default();
    return HttpResponse::Ok().json(ans);
}

/// Get the number of chapters in a book
#[utoipa::path(
    get,
    tag = "Info",
    path = "/chaptercount/{book}",
    responses(
        (status = 200, description = "Number of chapters and verses in a book", body = Count),
    ),
)]
#[get("/chaptercount/{book}")]
pub async fn get_chaptercount_book(
    path: web::Path<String>,
) -> HttpResponse {
    let book = path.into_inner();
    let mut index: i64 = -1;
    if let Some(pos) = constants::BOOKS.iter().position(|&b| b == book) {
        index = pos as i64;
    }
    if let Some(pos) = constants::ABBREVIATIONS.iter().position(|&ab| ab == book.to_uppercase()) {
        index = pos as i64;
    }
    if index == -1 {
        return HttpResponse::BadRequest().json(json!({
            "error": format!("Invalid URL parameter: {}", book)
        }));
    }
    let count = Count {
        chapters: constants::CHAPTERCOUNT[index as usize] as i64,
        verses: constants::VERSECOUNT[index as usize] as i64,
    };
    return HttpResponse::Ok().json(count);
}

/// Get information about a specific translation
#[utoipa::path(
    get,
    tag = "Info",
    path = "/{translation}/info",
    responses(
        (status = 200, description = "Get information about specific translation", body = TranslationInfo),
    ),
)]
#[get("/{translation}/info")]
pub async fn get_translation_info(
    app_data: web::Data<AppData>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let translation = path.into_inner().to_uppercase();
    let q = sqlx::query_as!(
        TranslationInfo,
        r#"
            SELECT
                t.name,
                l.lname AS language,
                t.full_name,
                t.seo_name,
                t.regional_name,
                ottn.otname AS ot_name,
                nttn.ntname AS nt_name,
                t.year,
                t.license,
                t.description
            FROM "Translation" t
            JOIN (
                SELECT
                    id,
                    name AS lname
                FROM "Language"
            ) l
                ON l.id = t.language_id
            JOIN (
                SELECT
                    translation_id,
                    name AS otname
                FROM "TestamentName"
                WHERE testament = 'OldTestament'
            ) ottn
                ON ottn.translation_id = t.id
            JOIN (
                SELECT
                    translation_id,
                    name AS ntname
                FROM "TestamentName"
                WHERE testament = 'NewTestament'
            ) nttn
                ON nttn.translation_id = t.id
            WHERE
                t.name = $1
        "#,
        &translation
    )
    .fetch_one(&app_data.pool)
    .await?;
    return Ok(HttpResponse::Ok().json(q));
}

/// Get a list of books with respect to the translation
///
/// The name of the book in the translation language, etc
#[utoipa::path(
    get,
    tag = "Info",
    path = "/{translation}/books",
    responses(
        (status = 200, description = "Get list of books with respect to the translation", body = Book),
    ),
)]
#[get("/{translation}/books")]
pub async fn get_translation_books(
    app_data: web::Data<AppData>,
    path: web::Path<String>,
) -> HttpResponse {
    let translation = path.into_inner().to_uppercase();
    let q = sqlx::query_as!(
        Book,
        r#"
            SELECT
                b.id AS book_id,
                b.abbreviation AS abbreviation,
                tb.name AS book_name,
                b.name AS book,
                b.testament AS "testament: Testament",
                tn.name AS testament_name
            FROM "Book" b
            JOIN "TestamentName" tn
                ON b.testament = tn.testament
            JOIN "Translation" t
                ON t.id = tn.translation_id
            JOIN "TranslationBookName" tb
                ON tb.translation_id = t.id
            AND b.id = tb.book_id
            WHERE
                t.name = $1
            ORDER BY
                b.id;
        "#,
        &translation
    )
    .fetch_all(&app_data.pool)
    .await
    .unwrap();
    if q.is_empty() {
        return HttpResponse::BadRequest().json(json!(format!(
            "The requested translation {} is not found on the server",
            &translation
        )));
    }
    return HttpResponse::Ok().json(q);
}

/// Get verses based on text search
///
/// If the length of the search text is less than 3, an empty array is returned. (Not errored as the frontend does not have good error handling).
/// Possible values for testament are old,new,ot,nt,old_testament,new_testament
#[utoipa::path(
    post,
    tag = "Verse",
    path = "/search",
    request_body = SearchParameters,
    responses(
        (status = 200, description = "Search throughout the bible", body = Verse),
    ),
)]
#[post("/search")]
pub async fn search(
    search_parameters: web::Json<SearchParameters>,
    app_data: web::Data<AppData>,
) -> HttpResponse {
    if search_parameters.search_text.len() < 3 {
        return HttpResponse::Ok().json(Vec::<String>::new());
    }

    let match_case = search_parameters.match_case.unwrap_or(false);
    let whole_words = search_parameters.whole_words.unwrap_or(false);

    // Step 1: Build the CTE ("base") for the main translation
    let mut base_qb: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
        WITH base AS (
            SELECT id, book, chapter, verse_number
            FROM fulltable
            WHERE verse "#,
    );

    if whole_words {
        if match_case {
            base_qb.push("~ ");
        } else {
            base_qb.push("~* ");
        }
        let actual_search_string = format!(r#"\m{}\M"#, &search_parameters.search_text.trim());
        base_qb.push_bind(actual_search_string);
    } else {
        if match_case {
            base_qb.push("like ");
        } else {
            base_qb.push("ilike ");
        }
        let actual_search_string = format!("%{}%", &search_parameters.search_text.trim());
        base_qb.push_bind(actual_search_string);
    }

    base_qb.push(" AND translation=");
    base_qb.push_bind(search_parameters.translation.to_uppercase());

    if let Some(books) = &search_parameters.books {
        if !books.is_empty() {
            base_qb.push(" AND book IN (");
            let mut sep = base_qb.separated(", ");
            for book in books {
                sep.push_bind(book);
            }
            base_qb.push(")");
        }
    }
    if let Some(abbreviations) = &search_parameters.abbreviations {
        if !abbreviations.is_empty() {
            base_qb.push(" AND abbreviation IN (");
            let mut sep = base_qb.separated(", ");
            for abbr in abbreviations {
                sep.push_bind(abbr.to_uppercase());
            }
            base_qb.push(")");
        }
    }
    base_qb.push(")");

    if let Some(testament) = &search_parameters.testament {
    let books: Vec<&'static str> = match testament.as_str() {
        "old" | "ot" | "old_testament" => constants::BOOKS[..39].to_vec(),
        "new" | "nt" | "new_testament" => constants::BOOKS[39..].to_vec(),
        _ => Vec::new(),
    };

    if !books.is_empty() {
        base_qb.push(" AND book IN (");
        let mut sep = base_qb.separated(", ");
        for book in books {
            sep.push_bind(book);
        }
        base_qb.push(")");
    }
}

    // Step 2: Query all requested translations (base + parallel)
    base_qb.push(
        r#"
        SELECT f.translation, f.book, f.abbreviation, f.book_name,
               f.chapter, f.verse_number, f.verse
        FROM fulltable f
        JOIN base b
          ON f.book = b.book
         AND f.chapter = b.chapter
         AND f.verse_number = b.verse_number
        WHERE f.translation IN ("#,
    );

    // Always include the base translation
    let mut separated = base_qb.separated(", ");
    separated.push_bind(search_parameters.translation.to_uppercase());

    // Add parallel translations if provided
    if let Some(parallels) = &search_parameters.parallel_translations {
        for t in parallels {
            separated.push_bind(t.to_uppercase());
        }
    }
    base_qb.push(")");

    base_qb.push(
        r#"
        ORDER BY b.id,
                 CASE f.translation
                    WHEN "#,
    );
    base_qb.push_bind(search_parameters.translation.to_uppercase());
    base_qb.push(" THEN 1 ELSE 2 END");

    let query = base_qb.build_query_as::<Verse>();
    let verses = query.fetch_all(&app_data.pool).await.unwrap();

    HttpResponse::Ok().json(verses)
}

/// Get the previous and next chapter / book to go to
///
/// The frontend needs to know what page lies before and
/// after a specific chapter. So, instead of making multiple
/// API calls, the information is sent in a separate endpoint
#[utoipa::path(
    post,
    tag = "Frontend Helper",
    path = "/nav",
    request_body = PageIn,
    responses(
        (status = 200, description = "Returns info about the previous and next pages to navigate to", body = PrevNext),
        (status = 400, description = "Atleast one argument of book or abbreviation is required",),
    ),
)]
#[post("/nav")]
pub async fn get_next_page(
    current_page: web::Json<PageIn>,
    app_data: web::Data<AppData>,
) -> Result<HttpResponse, AppError> {
    if current_page.book.is_none() && current_page.abbreviation.is_none() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "message": "Either one of book or abbreviation is required"
        })));
    }
    let previous: Option<PageOut>;
    let next: Option<PageOut>;
    let book_id;
    if let Some(ref x) = current_page.book {
        book_id = sqlx::query!(
            r#"
            SELECT id FROM "Book" WHERE name=$1
            "#,
            x
        )
        .fetch_one(&app_data.pool)
        .await?
        .id;
    } else {
        let abbreviation = current_page.abbreviation.clone().unwrap().to_uppercase();
        book_id = sqlx::query!(
            r#"
            SELECT id FROM "Book" WHERE abbreviation=$1
            "#,
            abbreviation
        )
        .fetch_one(&app_data.pool)
        .await?
        .id;
    }

    if current_page.chapter == 0 {
        if book_id == 1 {
            previous = None
        } else {
            let p = sqlx::query!(
                r#"
                SELECT name, abbreviation FROM "Book" where id=$1
                "#,
                book_id - 1
            )
            .fetch_one(&app_data.pool)
            .await?;
            previous = Some(PageOut {
                book: p.name,
                abbreviation: p.abbreviation,
                chapter: 0,
            });
        }
        if book_id == 66 {
            next = None
        } else {
            let n = sqlx::query!(
                r#"
                SELECT name, abbreviation FROM "Book" WHERE id=$1
                "#,
                book_id + 1
            )
            .fetch_one(&app_data.pool)
            .await?;
            next = Some(PageOut {
                book: n.name,
                abbreviation: n.abbreviation,
                chapter: 0,
            });
        }
        let prev_next = PrevNext { previous, next };
        return Ok(HttpResponse::Ok().json(prev_next));
    }

    if book_id == 1 && current_page.chapter == 1 {
        previous = None;
    } else {
        if current_page.chapter == 1 {
            let previous_chapter_count = sqlx::query!(
                r#"
                SELECT COUNT(*) AS count FROM "Chapter" WHERE book_id=$1
                "#,
                book_id - 1
            )
            .fetch_one(&app_data.pool)
            .await?
            .count
            .unwrap();
            let previous_book = sqlx::query!(
                r#"
                SELECT name, abbreviation FROM "Book" WHERE id=$1
                "#,
                book_id - 1
            )
            .fetch_one(&app_data.pool)
            .await?;
            previous = Some(PageOut {
                book: previous_book.name,
                abbreviation: previous_book.abbreviation,
                chapter: previous_chapter_count,
            });
        } else {
            let prev = sqlx::query!(
                r#"
                SELECT name, abbreviation FROM "Book" WHERE id=$1
                "#,
                book_id
            )
            .fetch_one(&app_data.pool)
            .await?;
            previous = Some(PageOut {
                book: prev.name,
                abbreviation: prev.abbreviation,
                chapter: current_page.chapter - 1,
            });
        }
    }

    if book_id == 66 && current_page.chapter == 22 {
        next = None;
    } else {
        let current_book_length = sqlx::query!(
            r#"
            SELECT COUNT(*) FROM "Chapter" WHERE book_id=$1
            "#,
            book_id
        )
        .fetch_one(&app_data.pool)
        .await?
        .count
        .unwrap();
        if current_page.chapter == current_book_length {
            let next_book = sqlx::query!(
                r#"
                SELECT name, abbreviation FROM "Book" WHERE id=$1
                "#,
                book_id + 1
            )
            .fetch_one(&app_data.pool)
            .await?;
            next = Some(PageOut {
                book: next_book.name,
                abbreviation: next_book.abbreviation,
                chapter: 1,
            })
        } else {
            let bo = sqlx::query!(
                r#"
                SELECT name, abbreviation FROM "Book" WHERE id=$1
                "#,
                book_id
            )
            .fetch_one(&app_data.pool)
            .await?;
            next = Some(PageOut {
                book: bo.name,
                abbreviation: bo.abbreviation,
                chapter: current_page.chapter + 1,
            });
        }
    }
    let prev_next = PrevNext { previous, next };

    return Ok(HttpResponse::Ok().json(prev_next));
}
