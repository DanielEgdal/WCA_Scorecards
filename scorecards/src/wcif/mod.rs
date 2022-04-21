///Module for creating data structure from json file in wcif format
pub mod json;
mod oauth;

use std::collections::HashMap;

use json::*;

use crate::pdf::scorecard::TimeLimit;

pub fn get_scorecard_info_for_round(id: &str, event: &str, round: usize) -> (Vec<usize>, HashMap<usize, String>, TimeLimit<'static>, String) {
    let json = oauth::get_wcif(id);

    let wcif = json::parse(json);  
    
    let id_map = get_id_map(&wcif.persons);

    let activity_id = format!("{}-r{}", event, round - 1);

    let round_json = wcif.events.iter().map(|event|event.rounds.iter()).flatten().find(|round| round.id == activity_id).unwrap();

    let advancement_ids = get_advancement_ids(&round_json, &round_json.advancement_condition);

    (advancement_ids, id_map, TimeLimit::Single(60000), wcif.name)
}

fn get_advancement_amount(round: &Round, advancement_condition: &Option<AdvancementCondition>) -> Option<usize> {
    let number_of_competitors = round.results.len();
    match advancement_condition {
        None => None,
        Some(v) => Some( match v {
            AdvancementCondition::Percent(level) => number_of_competitors * level / 100,
            AdvancementCondition::Ranking(level) => *level,
            AdvancementCondition::AttemptResult(level) => {
                let x = round.results.iter().enumerate().find(|(_, result)|{
                    match result.average {
                        -1 => true,
                        average => average as usize > *level
                    }
                }).map(|(x, _)| x);
                let percent = get_advancement_amount(round, &Some(AdvancementCondition::Percent(75))).unwrap();
                match x {
                    Some(v) if v < percent => v,
                    _ => percent
                }
            }
        })
    }
}

fn get_id_map(persons: &Vec<Person>) -> HashMap<usize, String> {
    let mut map = HashMap::new();
    persons.iter().for_each(|p|if let Some(v) = p.registrant_id { map.insert(v, p.name.clone()); });
    map
}

fn get_advancement_ids(round: &Round, advancement_condition: &Option<AdvancementCondition>) -> Vec<usize> {
    let advancement_amount = get_advancement_amount(round, advancement_condition);
    match advancement_amount {
        None => return vec![],
        Some(advancement_amount) => {
            let filtered = round.results.iter().filter(|result| result.ranking <= advancement_amount).collect::<Vec<_>>();
            if filtered.len() > advancement_amount {
                let not_included = filtered.last().unwrap().ranking;
                return filtered.iter().filter(|result| result.ranking != not_included).map(|result| result.person_id).collect();
            }
            filtered.iter().map(|result| result.person_id).collect()
        }
    }
}