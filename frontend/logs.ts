import { set_logger, set_log_level } from '../backend/pkg';

export class Logs {
    history = [];

    constructor () {
        set_logger((log) => this.history.push(log));
    }

    disable () {
        set_log_level('OFF');
    }

    enable () {
        set_log_level('TRACE');
    }
}
