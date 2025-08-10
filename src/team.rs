use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self},
    path::PathBuf,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rank {
    #[serde(rename = "Owner")]
    Owner,
    #[serde(rename = "Admin")]
    Admin,
    #[serde(rename = "Member")]
    Member,
}

impl Default for Rank {
    fn default() -> Self {
        Rank::Member
    }
}

impl Rank {
    pub fn from_str(s: &str) -> Rank {
        match s.to_ascii_lowercase().as_str() {
            "owner" => Rank::Owner,
            "admin" => Rank::Admin,
            _ => Rank::Member,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Rank::Owner => "Owner",
            Rank::Admin => "Admin",
            Rank::Member => "Member",
        }
    }
    pub fn sort_key(&self) -> u8 {
        match self {
            Rank::Owner => 0,
            Rank::Admin => 1,
            Rank::Member => 2,
        }
    }
    #[allow(dead_code)]
    pub fn emoji(&self) -> &'static str {
        match self {
            Rank::Owner => "👑",
            Rank::Admin => "🛡️",
            Rank::Member => "👤",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TeamMember {
    pub ign: String,
    pub country: String,
    pub skill: String,
    pub about: String,
    pub discord_tag: String,
    #[serde(default)]
    pub rank: Rank,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub name: String,
    pub members: Vec<TeamMember>,
}

impl Default for Team {
    fn default() -> Self {
        Team {
            name: "My Team".into(),
            members: vec![],
        }
    }
}

fn store_path() -> PathBuf {
    if let Ok(p) = std::env::var("TEAM_STORE_PATH") {
        return PathBuf::from(p);
    }
    PathBuf::from("team_data.json")
}

pub fn load() -> Team {
    let path = store_path();
    if !path.exists() {
        return Team::default();
    }
    match fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => Team::default(),
    }
}

pub fn save(team: &Team) -> io::Result<()> {
    let path = store_path();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            let _ = fs::create_dir_all(parent);
        }
    }
    let json = serde_json::to_string_pretty(team).unwrap_or_else(|_| "{}".into());
    fs::write(path, json)
}

pub fn set_name(new_name: &str) -> io::Result<Team> {
    let mut team = load();
    team.name = new_name.trim().to_string();
    save(&team)?;
    Ok(team)
}

pub fn upsert_member(member: TeamMember) -> io::Result<(Team, bool)> {
    let mut team = load();
    let mut updated = false;
    if let Some(existing) = team
        .members
        .iter_mut()
        .find(|m| m.ign.eq_ignore_ascii_case(&member.ign))
    {
        *existing = member;
        updated = true;
    } else {
        team.members.push(member);
    }
    save(&team)?;
    Ok((team, updated))
}

pub fn remove_member(ign: &str) -> io::Result<(Team, bool)> {
    let mut team = load();
    let orig_len = team.members.len();
    team.members.retain(|m| !m.ign.eq_ignore_ascii_case(ign));
    let removed = team.members.len() != orig_len;
    save(&team)?;
    Ok((team, removed))
}

fn flag_from_code(code: &str) -> Option<String> {
    if code.len() != 2 {
        return None;
    }
    let mut res = String::new();
    for ch in code.chars() {
        let u = ch.to_ascii_uppercase();
        if !u.is_ascii_alphabetic() {
            return None;
        }
        let base: u32 = 'A' as u32;
        let ri_offset: u32 = 0x1F1E6;
        let cp = ri_offset + (u as u32 - base);
        if let Some(c) = char::from_u32(cp) {
            res.push(c);
        }
    }
    Some(res)
}

pub fn country_flag(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }
    let trimmed = input.trim();
    if let Some(flag) = flag_from_code(trimmed) {
        return flag;
    }
    let name = trimmed.to_ascii_lowercase();
    let code = match name.as_str() {
        "belgium" | "belgie" | "belgië" => "BE",
        "netherlands" | "the netherlands" | "holland" | "nederland" => "NL",
        "united kingdom" | "uk" | "great britain" | "england" | "scotland" | "wales"
        | "northern ireland" => "GB",
        "united states" | "usa" | "us" | "america" => "US",
        "germany" | "deutschland" => "DE",
        "france" => "FR",
        "spain" | "españa" => "ES",
        "italy" | "italia" => "IT",
        "canada" => "CA",
        "australia" => "AU",
        "ireland" => "IE",
        "poland" => "PL",
        "portugal" => "PT",
        "sweden" => "SE",
        "norway" => "NO",
        "denmark" => "DK",
        "finland" => "FI",
        "luxembourg" => "LU",
        "switzerland" => "CH",
        "austria" => "AT",
        _ => "",
    };
    flag_from_code(code).unwrap_or_default()
}
