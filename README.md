# wof-stats: WOF Locker Allocation Scraper 🤖

This script scrapes a website's locker allocation table and saves it to a MySQL database. 

## Installation

1. Clone the repository or download the source code.
2. Navigate to the project's root directory.
3. Run `npm install` to install the required dependencies.

## Usage

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

## Credits

ChatGPT for generating this README ❤️🤖.
