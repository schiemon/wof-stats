# wof-stats: WOF Locker Allocation Scraper 🤖

This script scrapes a website's locker allocation table and saves it to a MySQL database. 

## Installation

1. Clone the repository or download the source code.
2. Navigate to the project's root directory.
3. Run `npm install` to install the required dependencies.

## Usage (`wof-stats-scraper`)

Create a .env file in the project's root directory with the following variables:

```
WOF_STATS_DB=your_database_name
WOF_STATS_TABLE=your_table_name
RDS_HOSTNAME=your_database_hostname
RDS_USERNAME=your_database_username
RDS_PASSWORD=your_database_password
RDS_PORT=your_database_port
WOF_STATS_URL=the_website_url_to_scrape
```

Modify the cron schedule expression ('*/15 * * * *' in the example code) to suit your needs.
Run `npm start` to start the script.

## Notes

wof-stats uses the following technologies:

- Node.js
- dotenv: Loads environment variables from a .env file
- cheerio: Parses HTML and provides a jQuery-like interface for manipulating it
- node-fetch: A light-weight module that brings window.fetch to Node.js
- mysql: A Node.js driver for MySQL databases
- node-cron: A module that allows you to schedule cron jobs to run at specific times

The script creates a new MySQL table if the specified table does not exist. It saves the current date and time along with the HTML of the website's locker allocation table to the database.

## Usage (`wof-stats-parser`)

First, navigate to the `parser` folder and install `rustup` as described [here](https://www.rust-lang.org/tools/install):

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

After that, install the nightly toolchain:

```
rustup toolchain install nightly
```

Finally you can run `wof-stats-parser`:

```
cargo run ./data/wof_stats_raw.json ./data/wof_stats.csv
```

This will build `wof-stats-parser`, and parses the stat entries in `./data/wof_stats_raw.json` and save the parsed data to `./data/wof_stats.csv`.

The parser expects the following format for the input file:

```
[
    {
        "id": 1,
        "version_date": "2023-01-01 07:00:00",
        "html": "<table>...</table>"
    },
    {
        "id": 2,
        "version_date": "2023-01-01 07:15:00",
        "html": "<table>...</table>"
    },
    ...
]
```

It produces the following output:

```
Date,Studio,NumAllocatedLockers
2023-01-01T07:00:00.000000000+0000,WOF 1 - Aachen Zentrum,6
2023-01-01T07:00:00.000000000+0000,WOF 2 - Würselen,8
...
2023-01-01T07:00:15.000000000+0000,WOF 1 - Aachen Zentrum,6
2023-01-01T07:00:15.000000000+0000,WOF 2 - Würselen,8
...
```

## Credits

ChatGPT and GitHub Copilot for generating this README ❤️🤖.
