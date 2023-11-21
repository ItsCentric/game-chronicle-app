export namespace main {
	
	export class Log {
	    title: string;
	    rating: number;
	    notes: string;
	    // Go type: LogStatus
	    status: any;
	    statusId: string;
	    finished: number;
	    timePlayed: number;
	
	    static createFrom(source: any = {}) {
	        return new Log(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.title = source["title"];
	        this.rating = source["rating"];
	        this.notes = source["notes"];
	        this.status = this.convertValues(source["status"], null);
	        this.statusId = source["statusId"];
	        this.finished = source["finished"];
	        this.timePlayed = source["timePlayed"];
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}
	export class TimePlayed {
	    hours: number;
	    minutes: number;
	
	    static createFrom(source: any = {}) {
	        return new TimePlayed(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.hours = source["hours"];
	        this.minutes = source["minutes"];
	    }
	}
	export class LogData {
	    title: string;
	    rating: number;
	    notes: string;
	    status: string;
	    finished: number;
	    timePlayed: TimePlayed;
	
	    static createFrom(source: any = {}) {
	        return new LogData(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.title = source["title"];
	        this.rating = source["rating"];
	        this.notes = source["notes"];
	        this.status = source["status"];
	        this.finished = source["finished"];
	        this.timePlayed = this.convertValues(source["timePlayed"], TimePlayed);
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}

}

