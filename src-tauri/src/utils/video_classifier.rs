//! Classifier - New media classification logic
//! 
//! This classifier implements a simplified approach to media classification:
//! 1. First check for extras (folder names or filename suffixes)
//! 2. Then check for numbered TV episodes (SxEy format or season folder + Ey)
//! 3. Then check for air date based TV shows (date patterns)
//! 4. Then check for movies (title with year in parentheses or dots)
//! 5. Everything else is generic

use regex::Regex;
use std::collections::HashMap;
use once_cell::sync::Lazy;

// ---------- Regex patterns ----------

// TV numbered patterns
static TV_SXXEYY: Lazy<Regex> = Lazy::new(|| Regex::new(
    r"(?i)S(\d{1,3})E(\d{1,4})(?:-E?(\d{1,4}))?"
).unwrap());

static TV_EYY: Lazy<Regex> = Lazy::new(|| Regex::new(
    r"(?i)E(\d{1,4})(?:-(\d{1,4}))?"
).unwrap());

static TV_EPYY: Lazy<Regex> = Lazy::new(|| Regex::new(
    r"(?i)Ep(\d{1,4})(?:-(\d{1,4}))?"
).unwrap());

// Season folder pattern
static SEASON_FOLDER: Lazy<Regex> = Lazy::new(|| Regex::new(
    r"(?i)^season\s+(\d+)$"
).unwrap());

// Date patterns
static DATE_ISO: Lazy<Regex> = Lazy::new(|| Regex::new(
    r"(\d{4})[-.](\d{1,2})[-.](\d{1,2})"
).unwrap());

static DATE_DMY: Lazy<Regex> = Lazy::new(|| Regex::new(
    r"(\d{1,2})[-.](\d{1,2})[-.](\d{4})"
).unwrap());

// Movie year patterns
static MOVIE_YEAR_PARENS: Lazy<Regex> = Lazy::new(|| Regex::new(
    r"(.+?)\s*\((\d{4})\)"
).unwrap());

static MOVIE_YEAR_DOTS: Lazy<Regex> = Lazy::new(|| Regex::new(
    r"(.+?)\.(\d{4})"
).unwrap());

// Version patterns
static VERSION_BRACES: Lazy<Regex> = Lazy::new(|| Regex::new(
    r"\{edition-(.+?)\}"
).unwrap());

static VERSION_DASH: Lazy<Regex> = Lazy::new(|| Regex::new(
    r"\s*-\s*([^-]+?)(?:\s*-\s*|$)"
).unwrap());

static VERSION_BRACKETS: Lazy<Regex> = Lazy::new(|| Regex::new(
    r"\s*-\s*\[([^\]]+)\]"
).unwrap());

// Part patterns
static PART_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(
    r"(?i)\s*-\s*\{?(cd|dvd|part|pt|disc|disk)(\d+)\}?"
).unwrap());

// External ID patterns - handles both imdb/imdbid variants
static EXTERNAL_ID: Lazy<Regex> = Lazy::new(|| Regex::new(
    r"[\[{](imdb|tmdb|tvdb)(?:id)?[:\- ]([^\]\}]+)[\]\}]"
).unwrap());

// ---------- Data structures ----------

#[derive(Debug, Clone, PartialEq)]
pub enum MediaType {
    Extra,
    TvEpisode,
    Movie,
    Generic,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExtraInfo {
    pub path: String,
    pub extra_type: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TvEpisodeInfo {
    pub show_name: String,
    pub source_path: String,
    pub season: i32,
    pub episode: i32,
    pub title: Option<String>,
    pub ep_end: Option<i32>,
    pub air_date: Option<String>,
    pub year: Option<i32>,
    pub part: Option<i32>,
    pub version: Option<String>,
    pub external_ids: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MovieInfo {
    pub title: String,
    pub source_path: String,
    pub year: Option<i32>,
    pub part: Option<i32>,
    pub version: Option<String>,
    pub external_ids: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericInfo {
    pub title: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MovieExtra {
    pub title: String,
    pub source_path: String,
    pub extra_type: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShowExtra {
    pub title: String,
    pub source_path: String,
    pub season: Option<i32>,
    pub episode: Option<i32>,
    pub extra_type: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassificationResult {
    pub media_type: MediaType,
    pub extra: Option<ExtraInfo>,
    pub tv_episode: Option<TvEpisodeInfo>,
    pub movie: Option<MovieInfo>,
    pub generic: Option<GenericInfo>,
}

// ---------- Main classification function ----------

pub fn classify_path(full_path: &str) -> ClassificationResult {
    let path_parts = parse_path(full_path);
    
    // 1. Check for extras first
    if let Some(extra) = detect_extra(&path_parts) {
        return ClassificationResult {
            media_type: MediaType::Extra,
            extra: Some(extra),
            tv_episode: None,
            movie: None,
            generic: None,
        };
    }
    
    // 2. Check for numbered TV episodes
    if let Some(tv) = detect_numbered_tv(&path_parts, full_path) {
        return ClassificationResult {
            media_type: MediaType::TvEpisode,
            extra: None,
            tv_episode: Some(tv),
            movie: None,
            generic: None,
        };
    }
    
    // 3. Check for air date based TV shows
    if let Some(tv) = detect_date_tv(&path_parts, full_path) {
        return ClassificationResult {
            media_type: MediaType::TvEpisode,
            extra: None,
            tv_episode: Some(tv),
            movie: None,
            generic: None,
        };
    }
    
    // 4. Check for movies
    if let Some(movie) = detect_movie(&path_parts, full_path) {
        return ClassificationResult {
            media_type: MediaType::Movie,
            extra: None,
            tv_episode: None,
            movie: Some(movie),
            generic: None,
        };
    }
    
    // 5. Everything else is generic
    ClassificationResult {
        media_type: MediaType::Generic,
        extra: None,
        tv_episode: None,
        movie: None,
        generic: Some(GenericInfo {
            title: path_parts.filename.clone(),
        }),
    }
}

// ---------- Path parsing ----------

#[derive(Debug, Clone)]
struct PathParts {
    folders: Vec<String>,
    filename: String,
    stem: String,
}

fn parse_path(full_path: &str) -> PathParts {
    let normalized = full_path.replace('\\', "/");
    let parts: Vec<&str> = normalized.split('/').collect();
    
    let filename = if let Some(last) = parts.last() {
        last.to_string()
    } else {
        String::new()
    };
    let folders: Vec<String> = parts[..parts.len()-1].iter()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    
    let stem = filename.rsplit_once('.')
        .map(|(s, _)| s.to_string())
        .unwrap_or(filename.clone());
    
    PathParts { folders, filename, stem }
}

// ---------- Extra detection ----------

fn detect_extra(path_parts: &PathParts) -> Option<ExtraInfo> {
    // Check folder names (exact match, case insensitive)
    let extra_folders = [
        ("behind the scenes", "behindthescenes"),
        ("deleted scenes", "deleted"),
        ("interviews", "interview"),
        ("scenes", "scene"),
        ("samples", "sample"),
        ("shorts", "short"),
        ("featurettes", "featurette"),
        ("clips", "clip"),
        ("others", "other"),
        ("extras", "extra"),
        ("trailers", "trailer")
    ];
    
    for folder in &path_parts.folders {
        for &(folder_name, extra_type) in &extra_folders {
            if folder.to_lowercase() == folder_name.to_lowercase() {
                return Some(ExtraInfo {
                    path: format!("/{}/{}", path_parts.folders.join("/"), path_parts.filename),
                    extra_type: extra_type.to_string(),
                });
            }
        }
    }
    
    // Check filename suffixes (exact match within string)
    let extra_suffixes = [
        ("-behindthescenes", "behindthescenes"),
        ("-deleted", "deleted"),
        ("-featurette", "featurette"),
        ("-interview", "interview"),
        ("-scene", "scene"),
        ("-short", "short"),
        ("-trailer", "trailer"),
        ("-other", "other")
    ];
    
    for &(suffix, extra_type) in &extra_suffixes {
        if path_parts.stem.to_lowercase().contains(suffix) {
            return Some(ExtraInfo {
                path: format!("/{}/{}", path_parts.folders.join("/"), path_parts.filename),
                extra_type: extra_type.to_string(),
            });
        }
    }
    
    None
}

// ---------- TV episode detection ----------

fn detect_numbered_tv(path_parts: &PathParts, original_path: &str) -> Option<TvEpisodeInfo> {
    // Check for SxEy format in filename
    if let Some(caps) = TV_SXXEYY.captures(&path_parts.stem) {
        let season = caps.get(1)?.as_str().parse::<i32>().ok()?;
        let episode = caps.get(2)?.as_str().parse::<i32>().ok()?;
        let ep_end = caps.get(3).and_then(|m| m.as_str().parse::<i32>().ok());
        
        let source_path = find_source_path(&path_parts.folders, original_path);
        let show_name = extract_show_name(&path_parts.folders, &path_parts.stem);
        
        let mut result = TvEpisodeInfo {
            show_name,
            source_path: source_path.clone(),
            season,
            episode,
            title: None,
            ep_end,
            air_date: None,
            year: None,
            part: None,
            version: None,
            external_ids: HashMap::new(),
        };
        
        // Parse version and part after episode number
        parse_version_title_and_part_after_episode(&path_parts.stem, &mut result);
        result.external_ids = parse_external_ids(&path_parts.stem);
        
        println!("TODO: Finished parsing TV episode source folder: {}", source_path);
        return Some(result);
    }
    
    // Check for season folder + Ey/Epy format
    if let Some(season_folder_idx) = find_season_folder(&path_parts.folders) {
        let season = extract_season_from_folder(&path_parts.folders[season_folder_idx]);
        
        // Check for Ey or Epy in filename
        if let Some(caps) = TV_EYY.captures(&path_parts.stem) {
            let episode = caps.get(1)?.as_str().parse::<i32>().ok()?;
            let ep_end = caps.get(2).and_then(|m| m.as_str().parse::<i32>().ok());
            
            let source_path = find_source_path(&path_parts.folders, original_path);
            let show_name = extract_show_name(&path_parts.folders, &path_parts.stem);
            
            let mut result = TvEpisodeInfo {
                show_name,
                source_path: source_path.clone(),
                season,
                episode,
                title: None,
                ep_end,
                air_date: None,
                year: None,
                part: None,
                version: None,
                external_ids: HashMap::new(),
            };
            
            parse_version_title_and_part_after_episode(&path_parts.stem, &mut result);
            result.external_ids = parse_external_ids(&path_parts.stem);
            
            println!("TODO: Finished parsing TV episode source folder: {}", source_path);
            return Some(result);
        }
        
        if let Some(caps) = TV_EPYY.captures(&path_parts.stem) {
            let episode = caps.get(1)?.as_str().parse::<i32>().ok()?;
            let ep_end = caps.get(2).and_then(|m| m.as_str().parse::<i32>().ok());
            
            let source_path = find_source_path(&path_parts.folders, original_path);
            let show_name = extract_show_name(&path_parts.folders, &path_parts.stem);
            
            let mut result = TvEpisodeInfo {
                show_name,
                source_path: source_path.clone(),
                season,
                episode,
                title: None,
                ep_end,
                air_date: None,
                year: None,
                part: None,
                version: None,
                external_ids: HashMap::new(),
            };
            
            parse_version_title_and_part_after_episode(&path_parts.stem, &mut result);
            result.external_ids = parse_external_ids(&path_parts.stem);
            
            println!("TODO: Finished parsing TV episode source folder: {}", source_path);
            return Some(result);
        }
    }
    
    // Check for specials folder (only immediate parent)
    if let Some(last_folder) = path_parts.folders.last() {
        if last_folder.to_lowercase() == "special" || last_folder.to_lowercase() == "specials" {
        
        // Check for Ey or Epy in filename
        if let Some(caps) = TV_EYY.captures(&path_parts.stem) {
            let episode = caps.get(1)?.as_str().parse::<i32>().ok()?;
            let ep_end = caps.get(2).and_then(|m| m.as_str().parse::<i32>().ok());
            
            let source_path = find_source_path(&path_parts.folders, original_path);
            let show_name = extract_show_name(&path_parts.folders, &path_parts.stem);
            
            let mut result = TvEpisodeInfo {
                show_name,
                source_path: source_path.clone(),
                season: 0, // Specials are season 0
                episode,
                title: None,
                ep_end,
                air_date: None,
                year: None,
                part: None,
                version: None,
                external_ids: HashMap::new(),
            };
            
            parse_version_title_and_part_after_episode(&path_parts.stem, &mut result);
            result.external_ids = parse_external_ids(&path_parts.stem);
            
            println!("TODO: Finished parsing TV episode source folder: {}", source_path);
            return Some(result);
        }
        
        if let Some(caps) = TV_EPYY.captures(&path_parts.stem) {
            let episode = caps.get(1)?.as_str().parse::<i32>().ok()?;
            let ep_end = caps.get(2).and_then(|m| m.as_str().parse::<i32>().ok());
            
            let source_path = find_source_path(&path_parts.folders, original_path);
            let show_name = extract_show_name(&path_parts.folders, &path_parts.stem);
            
            let mut result = TvEpisodeInfo {
                show_name,
                source_path: source_path.clone(),
                season: 0, // Specials are season 0
                episode,
                title: None,
                ep_end,
                air_date: None,
                year: None,
                part: None,
                version: None,
                external_ids: HashMap::new(),
            };
            
            parse_version_title_and_part_after_episode(&path_parts.stem, &mut result);
            result.external_ids = parse_external_ids(&path_parts.stem);
            
            println!("TODO: Finished parsing TV episode source folder: {}", source_path);
            return Some(result);
        }
        }
    }
    
    None
}

fn detect_date_tv(path_parts: &PathParts, original_path: &str) -> Option<TvEpisodeInfo> {
    // Check for date patterns in filename
    let date_match = DATE_ISO.captures(&path_parts.stem)
        .or_else(|| DATE_DMY.captures(&path_parts.stem))?;
    
    let (year, month, day) = if DATE_ISO.is_match(&path_parts.stem) {
        let year = date_match.get(1)?.as_str().parse::<i32>().ok()?;
        let month = date_match.get(2)?.as_str().parse::<i32>().ok()?;
        let day = date_match.get(3)?.as_str().parse::<i32>().ok()?;
        (year, month, day)
    } else {
        let day = date_match.get(1)?.as_str().parse::<i32>().ok()?;
        let month = date_match.get(2)?.as_str().parse::<i32>().ok()?;
        let year = date_match.get(3)?.as_str().parse::<i32>().ok()?;
        (year, month, day)
    };
    
    let air_date = format!("{:04}-{:02}-{:02}", year, month, day);
    
    // Calculate episode number as epoch days (days since 1970-01-01)
    let episode_number = days_since_epoch(year, month as u32, day as u32);
    
    let source_path = find_source_path(&path_parts.folders, original_path);
    let show_name = extract_show_name(&path_parts.folders, &path_parts.stem);
    
    // Check if there's a season folder
    let season = if let Some(season_folder_idx) = find_season_folder(&path_parts.folders) {
        extract_season_from_folder(&path_parts.folders[season_folder_idx])
    } else {
        year // Use year as season if no season folder
    };
    
    let mut result = TvEpisodeInfo {
        show_name,
        source_path: source_path.clone(),
        season,
        episode: episode_number as i32, // Use epoch days as episode number
        title: None,
        ep_end: None,
        air_date: Some(air_date),
        year: Some(year),
        part: None,
        version: None,
        external_ids: HashMap::new(),
    };
    
    parse_version_title_and_part_after_episode(&path_parts.stem, &mut result);
    result.external_ids = parse_external_ids(&path_parts.stem);
    
    println!("TODO: Finished parsing TV episode source folder: {}", source_path);
    Some(result)
}

// ---------- Movie detection ----------

fn detect_movie(path_parts: &PathParts, original_path: &str) -> Option<MovieInfo> {
    // Check for year in parentheses
    if let Some(caps) = MOVIE_YEAR_PARENS.captures(&path_parts.stem) {
        let title = caps.get(1)?.as_str().trim().to_string();
        let year = caps.get(2)?.as_str().parse::<i32>().ok()?;
        
        let source_path = find_source_path(&path_parts.folders, original_path);
        
        let mut result = MovieInfo {
            title,
            source_path: source_path.clone(),
            year: Some(year),
            part: None,
            version: None,
            external_ids: HashMap::new(),
        };
        
        parse_version_and_part_after_year(&path_parts.stem, &mut result);
        result.external_ids = parse_external_ids(&path_parts.stem);
        
        println!("TODO: Finished parsing movie source folder: {}", source_path);
        return Some(result);
    }
    
    // Check for year with dots
    if let Some(caps) = MOVIE_YEAR_DOTS.captures(&path_parts.stem) {
        let title = caps.get(1)?.as_str().trim().to_string();
        let year = caps.get(2)?.as_str().parse::<i32>().ok()?;
        
        let source_path = find_source_path(&path_parts.folders, original_path);
        
        let mut result = MovieInfo {
            title,
            source_path: source_path.clone(),
            year: Some(year),
            part: None,
            version: None,
            external_ids: HashMap::new(),
        };
        
        parse_version_and_part_after_year(&path_parts.stem, &mut result);
        result.external_ids = parse_external_ids(&path_parts.stem);
        
        println!("TODO: Finished parsing movie source folder: {}", source_path);
        return Some(result);
    }
    
    None
}

// ---------- Extra classification ----------

/// Classify an extra as a movie extra
pub fn classify_movie_extra(extra_info: &ExtraInfo, source_path: &str) -> Option<MovieExtra> {
    // Extract title from the path, removing extra suffixes
    let path_parts = parse_path(&extra_info.path);
    let mut title = path_parts.stem.clone();
    
    // Remove extra suffixes from title
    let extra_suffixes = [
        "-behindthescenes", "-deleted", "-featurette", "-interview",
        "-scene", "-short", "-trailer", "-other"
    ];
    
    for suffix in &extra_suffixes {
        if title.to_lowercase().ends_with(suffix) {
            title = title[..title.len() - suffix.len()].trim().to_string();
            break;
        }
    }
    
    Some(MovieExtra {
        title,
        source_path: source_path.to_string(),
        extra_type: extra_info.extra_type.clone(),
    })
}

/// Classify an extra as a show extra
pub fn classify_show_extra(extra_info: &ExtraInfo, source_path: &str) -> Option<ShowExtra> {
    // Extract title from the path, removing extra suffixes
    let path_parts = parse_path(&extra_info.path);
    let mut title = path_parts.stem.clone();
    
    // Remove extra suffixes from title
    let extra_suffixes = [
        "-behindthescenes", "-deleted", "-featurette", "-interview",
        "-scene", "-short", "-trailer", "-other"
    ];
    
    for suffix in &extra_suffixes {
        if title.to_lowercase().ends_with(suffix) {
            title = title[..title.len() - suffix.len()].trim().to_string();
            break;
        }
    }
    
    // Determine if this is for a specific season/episode
    let mut season = None;
    let mut episode = None;
    
    // Check if we're in a season/episode folder
    // Iterate in reverse order (deepest to shallowest) and limit to last 4 folders
    let folders_to_check = path_parts.folders.iter().rev().take(4);
    
    for folder in folders_to_check {
        // Check for SxEy or SxEpy patterns in folder names (prioritize episode patterns)
        if let Some(caps) = TV_SXXEYY.captures(folder) {
            season = caps.get(1).and_then(|m| m.as_str().parse::<i32>().ok());
            episode = caps.get(2).and_then(|m| m.as_str().parse::<i32>().ok());
            break;
        }
        
        // Check for season folder
        if let Some(caps) = SEASON_FOLDER.captures(folder) {
            season = caps.get(1).and_then(|m| m.as_str().parse::<i32>().ok());
            break;
        }
        
        // Check for specials folder (season 0)
        if folder.to_lowercase() == "special" || folder.to_lowercase() == "specials" {
            season = Some(0);
            break;
        }
    }
    
    Some(ShowExtra {
        title,
        source_path: source_path.to_string(),
        season,
        episode,
        extra_type: extra_info.extra_type.clone(),
    })
}

// ---------- Helper functions ----------

/// Returns the number of days since the Unix epoch (1970-01-01).
/// Dates before the epoch will return negative numbers.
fn days_since_epoch(year: i32, month: u32, day: u32) -> i64 {
    let epoch = chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
    let date = chrono::NaiveDate::from_ymd_opt(year, month, day)
        .expect("Invalid date");
    (date - epoch).num_days()
}

fn find_source_path(folders: &[String], original_path: &str) -> String {
    // Check if the original path was absolute (starts with "/")
    let is_absolute = original_path.starts_with('/');
    
    // Look for season folder and return its parent path
    if let Some(season_folder_idx) = find_season_folder(folders) {
        if season_folder_idx > 0 {
            let path = folders[..season_folder_idx].join("/");
            return if is_absolute { format!("/{}", path) } else { path };
        }
    }
    
    // Otherwise return the full path to the last folder (closest to file)
    let path = folders.join("/");
    if is_absolute { format!("/{}", path) } else { path }
}

fn find_season_folder(folders: &[String]) -> Option<usize> {
    // Only check the immediate parent folder (last folder)
    if let Some(last_folder) = folders.last() {
        if SEASON_FOLDER.is_match(last_folder) {
            return Some(folders.len() - 1);
        }
    }
    None
}

fn extract_season_from_folder(folder: &str) -> i32 {
    SEASON_FOLDER.captures(folder)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse::<i32>().ok())
        .unwrap_or(1)
}

fn extract_show_name(folders: &[String], stem: &str) -> String {
    // Try to find show name from folders first
    for folder in folders.iter().rev() {
        if !SEASON_FOLDER.is_match(folder) && 
           folder.to_lowercase() != "special" && 
           folder.to_lowercase() != "specials" {
            return folder.clone();
        }
    }
    
    // Fallback to stem with TV patterns removed
    let cleaned = TV_SXXEYY.replace_all(stem, "");
    let cleaned = TV_EYY.replace_all(&cleaned, "");
    let cleaned = TV_EPYY.replace_all(&cleaned, "");
    cleaned.trim().to_string()
}

fn parse_version_title_and_part_after_episode(stem: &str, tv_info: &mut TvEpisodeInfo) {
    // Find the episode pattern and parse everything after it
    let episode_pattern = if TV_SXXEYY.is_match(stem) {
        TV_SXXEYY.find(stem).map(|m| m.end())
    } else if TV_EYY.is_match(stem) {
        TV_EYY.find(stem).map(|m| m.end())
    } else if TV_EPYY.is_match(stem) {
        TV_EPYY.find(stem).map(|m| m.end())
    } else {
        None
    };
    
    if let Some(end_pos) = episode_pattern {
        let after_episode = &stem[end_pos..];
        parse_version_title_and_part_from_suffix_tv(after_episode, tv_info);
    }
}

fn parse_version_and_part_after_year(stem: &str, movie_info: &mut MovieInfo) {
    // Find the year pattern and parse everything after it
    let year_pattern = if MOVIE_YEAR_PARENS.is_match(stem) {
        MOVIE_YEAR_PARENS.find(stem).map(|m| m.end())
    } else if MOVIE_YEAR_DOTS.is_match(stem) {
        MOVIE_YEAR_DOTS.find(stem).map(|m| m.end())
    } else {
        None
    };
    
    if let Some(end_pos) = year_pattern {
        let after_year = &stem[end_pos..];
        parse_version_and_part_from_suffix_movie(after_year, movie_info);
    }
}

fn parse_version_title_and_part_from_suffix_tv(suffix: &str, tv_info: &mut TvEpisodeInfo) {
    // Parse version
    if let Some(caps) = VERSION_BRACES.captures(suffix) {
        if let Some(version_match) = caps.get(1) {
            tv_info.version = Some(version_match.as_str().to_string());
        }
    }
    if let Some(caps) = VERSION_DASH.captures(suffix) {
        if let Some(version_match) = caps.get(1) {
            //If version already set, then this is the title
            if tv_info.version.is_some() {
                tv_info.title = Some(version_match.as_str().to_string());
            } else {
                tv_info.version = Some(version_match.as_str().to_string());
            }
        }
    } else if let Some(caps) = VERSION_BRACKETS.captures(suffix) {
        if let Some(version_match) = caps.get(1) {
            tv_info.version = Some(version_match.as_str().to_string());
        }
    }
    
    // Parse part
    if let Some(caps) = PART_PATTERN.captures(suffix) {
        if let Some(part_match) = caps.get(2) {
            tv_info.part = part_match.as_str().parse::<i32>().ok();
        }
    }
}

fn parse_version_and_part_from_suffix_movie(suffix: &str, movie_info: &mut MovieInfo) {
    // Parse version
    if let Some(caps) = VERSION_BRACES.captures(suffix) {
        if let Some(version_match) = caps.get(1) {
            movie_info.version = Some(version_match.as_str().to_string());
        }
    } else if let Some(caps) = VERSION_DASH.captures(suffix) {
        if let Some(version_match) = caps.get(1) {
            movie_info.version = Some(version_match.as_str().to_string());
        }
    } else if let Some(caps) = VERSION_BRACKETS.captures(suffix) {
        if let Some(version_match) = caps.get(1) {
            movie_info.version = Some(version_match.as_str().to_string());
        }
    }
    
    // Parse part
    if let Some(caps) = PART_PATTERN.captures(suffix) {
        if let Some(part_match) = caps.get(2) {
            movie_info.part = part_match.as_str().parse::<i32>().ok();
        }
    }
}

fn parse_external_ids(text: &str) -> HashMap<String, String> {
    let mut ids = HashMap::new();
    
    for caps in EXTERNAL_ID.captures_iter(text) {
        if let (Some(id_type), Some(id_value)) = (caps.get(1), caps.get(2)) {
            ids.insert(id_type.as_str().to_lowercase(), id_value.as_str().to_string());
        }
    }
    
    ids
}

// ---------- Tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extra_folder_detection() {
        let result = classify_path("Movies/Avatar/Behind The Scenes/Making Of.mkv");
        assert_eq!(result.media_type, MediaType::Extra);
        assert!(result.extra.is_some());
    }

    #[test]
    fn test_extra_filename_suffix() {
        let result = classify_path("Movies/Avatar-trailer.mkv");
        assert_eq!(result.media_type, MediaType::Extra);
        assert!(result.extra.is_some());
    }

    #[test]
    fn test_tv_sxxeyy() {
        let result = classify_path("TV/Some Show/Season 1/Some.Show.S01E01.mkv");
        assert_eq!(result.media_type, MediaType::TvEpisode);
        let tv = result.tv_episode.unwrap();
        assert_eq!(tv.season, 1);
        assert_eq!(tv.episode, 1);
        assert_eq!(tv.show_name, "Some Show");
    }

    #[test]
    fn test_tv_season_folder_ey() {
        let result = classify_path("TV/Some Show/Season 2/E05.mkv");
        assert_eq!(result.media_type, MediaType::TvEpisode);
        let tv = result.tv_episode.unwrap();
        assert_eq!(tv.season, 2);
        assert_eq!(tv.episode, 5);
        assert_eq!(tv.show_name, "Some Show");
    }

    #[test]
    fn test_tv_specials() {
        let result = classify_path("TV/Some Show/Specials/E01.mkv");
        assert_eq!(result.media_type, MediaType::TvEpisode);
        let tv = result.tv_episode.unwrap();
        assert_eq!(tv.season, 0);
        assert_eq!(tv.episode, 1);
    }

    #[test]
    fn test_tv_date_based() {
        let result = classify_path("TV/News Show/2024-10-15.mkv");
        assert_eq!(result.media_type, MediaType::TvEpisode);
        let tv = result.tv_episode.unwrap();
        assert_eq!(tv.air_date, Some("2024-10-15".to_string()));
        assert_eq!(tv.season, 2024);
    }

    #[test]
    fn test_movie_year_parens() {
        let result = classify_path("Movies/Avatar (2009).mkv");
        assert_eq!(result.media_type, MediaType::Movie);
        let movie = result.movie.unwrap();
        assert_eq!(movie.title, "Avatar");
        assert_eq!(movie.year, Some(2009));
    }

    #[test]
    fn test_movie_year_dots() {
        let result = classify_path("Movies/Avatar.2009.mkv");
        assert_eq!(result.media_type, MediaType::Movie);
        let movie = result.movie.unwrap();
        assert_eq!(movie.title, "Avatar");
        assert_eq!(movie.year, Some(2009));
    }

    #[test]
    fn test_movie_with_version() {
        let result = classify_path("Movies/Avatar (2009) - Directors Cut.mkv");
        assert_eq!(result.media_type, MediaType::Movie);
        let movie = result.movie.unwrap();
        assert_eq!(movie.title, "Avatar");
        assert_eq!(movie.year, Some(2009));
        assert_eq!(movie.version, Some("Directors Cut".to_string()));
    }

    #[test]
    fn test_movie_with_part() {
        let result = classify_path("Movies/Avatar (2009) - part1.mkv");
        assert_eq!(result.media_type, MediaType::Movie);
        let movie = result.movie.unwrap();
        assert_eq!(movie.title, "Avatar");
        assert_eq!(movie.year, Some(2009));
        assert_eq!(movie.part, Some(1));
    }

    #[test]
    fn test_generic() {
        let result = classify_path("Videos/GoPro Mountain Run.mp4");
        assert_eq!(result.media_type, MediaType::Generic);
        let generic = result.generic.unwrap();
        assert_eq!(generic.title, "GoPro Mountain Run.mp4");
    }

    #[test]
    fn test_epoch_days_calculation() {
        // Test epoch days calculation
        assert_eq!(days_since_epoch(1970, 1, 1), 0);   // Unix epoch
        assert_eq!(days_since_epoch(1970, 1, 2), 1);   // Day after epoch
        assert_eq!(days_since_epoch(1969, 12, 31), -1); // Day before epoch
        assert_eq!(days_since_epoch(2024, 10, 15), 20011); // Future date
    }
}
