import chalk from "chalk";
import log from 'loglevel';
import prefix from "loglevel-plugin-prefix"

export function getLogger() {
    prefix.reg(log);
    const logger = log.getLogger("wof-stats");
    const colors = {
        TRACE: chalk.magenta,
        DEBUG: chalk.cyan,
        INFO: chalk.blue,
        WARN: chalk.yellow,
        ERROR: chalk.red,
    };
    prefix.apply(logger, {
        format(level, name, timestamp) {
            return `${chalk.gray(`[${timestamp}]`)} ${colors[level.toUpperCase()](level)} ${chalk.green(`${name}:`)}`;
        },
    });
    logger.enableAll()


    return logger;
}