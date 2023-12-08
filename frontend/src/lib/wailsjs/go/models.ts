export namespace main {
	
	export class AccessTokenResponse {
	    access_token: string;
	    expires_in: number;
	    token_type: string;
	
	    static createFrom(source: any = {}) {
	        return new AccessTokenResponse(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.access_token = source["access_token"];
	        this.expires_in = source["expires_in"];
	        this.token_type = source["token_type"];
	    }
	}
	export class SimplifiedIgdbCover {
	    id: number;
	    image_id?: string;
	
	    static createFrom(source: any = {}) {
	        return new SimplifiedIgdbCover(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.image_id = source["image_id"];
	    }
	}
	export class IgdbGame {
	    name: string;
	    cover: SimplifiedIgdbCover;
	
	    static createFrom(source: any = {}) {
	        return new IgdbGame(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.name = source["name"];
	        this.cover = this.convertValues(source["cover"], SimplifiedIgdbCover);
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
	export class Log {
	    title: string;
	    rating: number;
	    notes: string;
	    // Go type: LogStatus
	    status: any;
	    statusId: string;
	    // Go type: time
	    startedOn: any;
	    // Go type: time
	    finishedOn: any;
	    timePlayedMinutes: number;
	
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
	        this.startedOn = this.convertValues(source["startedOn"], null);
	        this.finishedOn = this.convertValues(source["finishedOn"], null);
	        this.timePlayedMinutes = source["timePlayedMinutes"];
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
	export class InsertGameLogResponse {
	    log: Log;
	    errors: {[key: string]: string};
	
	    static createFrom(source: any = {}) {
	        return new InsertGameLogResponse(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.log = this.convertValues(source["log"], Log);
	        this.errors = source["errors"];
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
	    notes?: string;
	    status: string;
	    // Go type: time
	    startedOn: any;
	    // Go type: time
	    finishedOn?: any;
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
	        this.startedOn = this.convertValues(source["startedOn"], null);
	        this.finishedOn = this.convertValues(source["finishedOn"], null);
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

