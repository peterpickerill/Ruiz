#[macro_use]
extern crate rocket;
use rocket::{fs::{relative, FileServer}, response::Redirect};
use std::{fs::File, cmp};

use rocket::State;
use rocket_dyn_templates::{context, Template};
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub meta: Meta,
    pub questions: Vec<Question>
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub title: String,
    pub background_image: Option<String>
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QuestionType {
    #[default]
    None,
    TopicIntro,
    Text,
    Audio,
    MultipleChoice,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Question {
    pub topic: String,
    pub question: Option<String>,
    pub image: Option<String>,
    #[serde(rename = "type")]
    pub type_field: QuestionType,
    pub answer: Option<String>,
    pub source: Option<String>,
    pub options: Option<Vec<String>>,
}

const QUESTION_TEMPLATE_NAME: &str = "home";
const QUIZ_TITLE_TEMPLATE_NAME: &str = "title";
const QUIZ_TOPIC_TEMPLATE_NAME: &str = "topic";
const QUIZ_END_TEMPLATE_NAME: &str = "end";

#[get("/")]
fn index(quiz: &State<Root>) -> Template {
    Template::render(QUIZ_TITLE_TEMPLATE_NAME, context! {
        title: quiz.meta.title.as_str(),
        meta: context! {
            title: quiz.meta.title.as_str(),
            start: uri!(show_question(1, false))
        }
    })   
}

#[get("/end")]
fn end(quiz: &State<Root>) -> Template {
    Template::render(QUIZ_END_TEMPLATE_NAME, context! {
        title: quiz.meta.title.as_str(),
        meta: context! {
            title: quiz.meta.title.as_str(),
            start: uri!(show_question(1, false))
        }
    })   
}

#[get("/question/<question_number>?<answer>")]
fn show_question(quiz: &State<Root>, question_number: usize, answer: bool) -> Template {
    let question_option = quiz.questions.get(question_number);
    let prev_question = cmp::max(question_number, 1) - 1 ;
    let next_question = question_number + 1;
    let real_question_number = quiz.questions.clone().into_iter().take(question_number).filter(|s| s.type_field != QuestionType::TopicIntro).count();

    let prev_link = match question_number {
        1 => uri!(index),
        _ => uri!(show_question(prev_question, answer))
    };

    if (question_option.is_none()) {
        return Template::render(QUIZ_END_TEMPLATE_NAME, context! {
            meta: context! {
                title: quiz.meta.title.as_str(),
                link: uri!(show_question(1, true))
            },
        }); 
    }
    let question = quiz.questions[question_number - 1].clone();

    let next_link = uri!(show_question(next_question, answer));

    if question.type_field == QuestionType::TopicIntro {
        return Template::render(QUIZ_TOPIC_TEMPLATE_NAME, context! {
            title: question.topic.as_str(),
            image: question.image,
            meta: context! {
                title: quiz.meta.title.as_str(),
                prev: prev_link,
                next: next_link,
                background: quiz.meta.background_image.as_ref().unwrap_or(&String::new())
            },
        });
    }

    let options = match question.options {
        Some(x) => x,
        None => vec!()
    };
    let image = match question.image {
        Some(x) => x,
        None => String::new()
    };
    let question_answer = match answer {
        true => question.answer.unwrap_or(String::new()),
        false => String::new()
    };
    Template::render(
        QUESTION_TEMPLATE_NAME,
        context! {
            meta: context! {
                title: quiz.meta.title.as_str(),
                prev: prev_link,
                next: next_link,
                background: quiz.meta.background_image.as_ref().unwrap_or(&String::new())
            },
            question: context! {
                number: real_question_number,
                topic: question.topic,
                text: question.question,
                image: image,
                options: options,
                answer: question_answer
            }
        },
    )
}

#[launch]
fn rocket() -> _ {
    let quiz_file = match File::open("data/questions.json") {
        Ok(x) => x,
        Err(_) => panic!("Cannot find quiz file"),
    };
    let quiz: Root = match serde_json::from_reader(quiz_file) {
        Ok(quiz) => quiz,
        Err(_) => panic!("Cannot read quiz file"),
    };
    rocket::build()
        .manage(quiz)
        .attach(Template::fairing())
        .mount("/static", FileServer::from(relative!("static")))
        .mount("/", routes![index, show_question, end])
}
