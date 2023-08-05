#![feature(iter_next_chunk)]
use chrono::{DateTime, FixedOffset};
use indicatif::ProgressBar;
use polars::prelude::*;
use scraper::{ElementRef, Html, Selector};
use serde::Deserialize;
use std::{env, fs::File};

struct WOFStatsEntry {
    date: DateTime<FixedOffset>,
    studio: String,
    // Number (in percent) of lockers allocated to this studio.
    num_allocated_lockers: u8,
}

mod wof_date_format {
    use chrono::{DateTime, FixedOffset, NaiveDateTime};
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        return NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
            .map(|ndt| {
                DateTime::<FixedOffset>::from_utc(ndt, FixedOffset::east_opt(2 * 3600).unwrap())
            })
            .map_err(serde::de::Error::custom);
    }
}

#[derive(Deserialize)]
struct WOFStatsPageSnapshot {
    id: i64,
    #[serde(with = "wof_date_format")]
    version_date: DateTime<FixedOffset>,
    html: String,
}

fn parse_studio_and_locker_allocation(
    wof_studio_stat_row: ElementRef,
) -> Result<(String, u8), String> {
    let [studio_cell, locker_allocation_cell] = wof_studio_stat_row
        .select(&Selector::parse("td").unwrap())
        .next_chunk::<2>()
        .unwrap();

    let studio = studio_cell.inner_html().trim().to_string();

    let locker_allocation = locker_allocation_cell
        .select(&Selector::parse("div > div").unwrap())
        .next()
        .ok_or("Could not extract locker allocation.")?
        .inner_html()
        .trim()
        .split('%')
        .next()
        .ok_or("Could not extract locker allocation number.")?
        .parse::<i16>()
        .map(|num_allocated_lockers| num_allocated_lockers.abs() as u8)
        .map_err(|e| {
            format!(
                "Could not parse locker allocation number ({}).",
                e.to_string()
            )
        })?;

    Ok((studio, locker_allocation))
}

// TODO: Decouple parsing from outputting.
fn parse_wof_stats(wof_stats_raw_json: String) -> Result<DataFrame, String> {
    const DATE_COLUMN_NAME: &str = "Date";
    const STUDIO_COLUMN_NAME: &str = "Studio";
    const NUM_ALLOCATED_LOCKERS_COLUMN_NAME: &str = "NumAllocatedLockers";

    let wof_stats_page_snapshots: Vec<WOFStatsPageSnapshot> =
        serde_json::from_str(&wof_stats_raw_json).unwrap();

    let mut wof_stats_entries: Vec<WOFStatsEntry> = Vec::new();
    let pb = ProgressBar::new(wof_stats_page_snapshots.len() as u64);
    println!("Parsing snapshots...");

    // Maybe putting everything in a struct and returning it?
    let mut num_parsed_stats_page_snapshots = 0;
    let mut num_stats_rows = 0;
    let mut num_parsed_stats_rows = 0;

    for wof_stats_page_snapshot in &wof_stats_page_snapshots {
        let wof_stats_html = Html::parse_fragment(wof_stats_page_snapshot.html.as_str());

        if let Some(wof_stats_table) = wof_stats_html
            .select(&Selector::parse("table").unwrap())
            .next()
        {
            let mut successfully_parsed_page_snapshot = true;

            for (row_num, wof_studio_stat_row) in wof_stats_table
                .select(&Selector::parse("tbody > tr").unwrap())
                .enumerate()
            {
                num_stats_rows += 1;

                match parse_studio_and_locker_allocation(wof_studio_stat_row) {
                    Ok((studio, num_allocated_lockers)) => {
                        wof_stats_entries.push(WOFStatsEntry {
                            date: wof_stats_page_snapshot.version_date,
                            studio,
                            num_allocated_lockers,
                        });
                        num_parsed_stats_rows += 1;
                    }
                    Err(e) => {
                        eprintln!(
                        "Skip parsing row {} of snapshot with id '{}' because of following error: {}",
                        row_num, wof_stats_page_snapshot.id, e
                    );
                        successfully_parsed_page_snapshot = false;
                    }
                }
            }

            if successfully_parsed_page_snapshot {
                num_parsed_stats_page_snapshots += 1;
            }
        } else {
            eprintln!(
                "Skipping snapshot with id '{}' because it does not contain a table.",
                wof_stats_page_snapshot.id
            );
        }

        pb.inc(1);
    }

    let utc_2 = Some("Europe/Berlin".to_string());
    let dates_foo: Vec<AnyValue> = wof_stats_entries
        .iter()
        .map(|entry| {
            AnyValue::Datetime(
                entry.date.timestamp_millis(),
                TimeUnit::Milliseconds,
                &utc_2,
            )
        })
        .collect();

    let dates: Series = Series::from_any_values_and_dtype(
        "Dates",
        &dates_foo,
        &DataType::Datetime(TimeUnit::Milliseconds, utc_2.clone()),
        true,
    )
    .unwrap();

    let studios: Series = wof_stats_entries
        .iter()
        .map(|entry| entry.studio.clone())
        .collect();

    let num_allocated_lockers: Series = wof_stats_entries
        .iter()
        .map(|entry| entry.num_allocated_lockers as u32)
        .collect();

    let df = polars::df!(
        DATE_COLUMN_NAME => &dates,
        STUDIO_COLUMN_NAME => &studios,
        NUM_ALLOCATED_LOCKERS_COLUMN_NAME => &num_allocated_lockers
    )
    .map_err(|e: PolarsError| e.to_string());

    // Print some statistics.
    println!(
        "Parsed {} of {} snapshots.",
        num_parsed_stats_page_snapshots,
        wof_stats_page_snapshots.len()
    );

    println!(
        "Parsed {} of {} rows.",
        num_parsed_stats_rows, num_stats_rows
    );

    return df;
}

fn main() {
    let mut args = env::args().skip(1);

    let input_path = args.next().expect("Input file not specified.");
    let output_path = args.next().expect("Output file not specified.");

    if !input_path.ends_with(".json") {
        panic!("The input file must be a JSON file.");
    }

    if !output_path.ends_with(".csv") {
        panic!("The output file will be a CSV file.");
    }

    let wof_stats_raw_json = std::fs::read_to_string(input_path).unwrap();

    let mut df = parse_wof_stats(wof_stats_raw_json).unwrap();

    CsvWriter::new(&mut File::create(output_path).unwrap())
        .has_header(true)
        .finish(&mut df)
        .unwrap();
}
