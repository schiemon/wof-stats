import { config } from "dotenv";
import { load } from 'cheerio';
import { getLogger } from "./logger.js";
import fetch from "node-fetch";
import mysql from "mysql";
import cron from "node-cron"
import { promisify } from "util";

const logger = getLogger();

let WOF_STATS_TABLE;
let WOF_STATS_URL;

const execQuery = async (fn) => {
    let connection = null;

    async function connect() {
        logger.debug("Connect...");
        return new Promise((res, rej) => {
            connection =
                mysql.createConnection({
                    host: process.env.RDS_HOSTNAME,
                    user: process.env.RDS_USERNAME,
                    database: process.env.WOF_STATS_DB,
                    password: process.env.RDS_PASSWORD,
                    port: process.env.RDS_PORT
                });

            connection.connect((err) => {
                if (err) {
                    rej(err);
                } else {
                    res();
                }
            })
        }).catch((err) => {
            logger.error("Could not establish connection.");
            connection = null;
            throw err;
        });
    }

    async function disconnect() {
        logger.debug("Disconnect...");
        return new Promise((res, rej) => {
            if (connection) {
                connection.end((err) => {
                    if (err) {
                        rej(err);
                    } else {
                        res();
                    }
                });
            } else {
                res();
            }
        })
            .then(() => {
                connection = null;
            })
            .catch(err => {
                logger.error("Could not close connection.");
                connection = null;
                throw err;
            })
    }

    try {
        await connect();
        const query = promisify(connection.query.bind(connection));
        await fn(query);
    } catch (err) {
        logger.error("Following error occurred while processing the query:");
        logger.error(err);
    } finally {
        await disconnect();
    }
}

function getCurrentDateForDB() {
    return new Date().toISOString().slice(0, 19).replace('T', ' ');
}

async function setup() {
    logger.info("Setup...")
    config();
    WOF_STATS_TABLE = process.env.WOF_STATS_TABLE;
    WOF_STATS_URL = process.env.WOF_STATS_URL;
    logger.info({
        host: process.env.RDS_HOSTNAME,
        user: process.env.RDS_USERNAME,
        database: process.env.WOF_STATS_DB,
        table: WOF_STATS_TABLE,
        url: WOF_STATS_URL,
        port: process.env.RDS_PORT
    })

    await execQuery(async query => {
        await query(`
            CREATE TABLE IF NOT EXISTS \`${WOF_STATS_TABLE}\` (
                \`id\` INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
                \`version_date\` DATETIME NOT NULL,
                \`html\` TEXT NOT NULL
            );
        `);
    })
}

async function getLockerAllocationTableSite() {
    const response = await fetch(WOF_STATS_URL);
    return await response.text();
}

function getLockerAllocationTable(html) {
    const $ = load(html);
    return $("table").toString();
}

async function save(tableHtml) {
    logger.info("Saving...");

    await execQuery(async (query) => {
        await query(`INSERT INTO \`${WOF_STATS_TABLE}\` (\`version_date\`, \`html\`) VALUES (?, ?)`, [getCurrentDateForDB(), tableHtml]);
    });
}

async function run() {
    logger.info("Running...");
    const html = await getLockerAllocationTableSite();
    const tableHtml = getLockerAllocationTable(html);
    await save(tableHtml)
}

await setup();

// Run every 15 minutes (snooze inclusive).
cron.schedule('* */15 * * * *', async () => {
    await run();
});