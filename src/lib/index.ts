import type { z } from "zod";
import type { NewLogFormSchema } from "./schemas";
import { main } from "./wailsjs/go/models";

export function logDataFromForm(igdbGame: main.IgdbGame, formData: z.infer<NewLogFormSchema>) {
    const timePlayed = new main.TimePlayed();
    const newLogData = new main.LogData();
    timePlayed.hours = formData.timePlayedHours ?? 0;
    timePlayed.minutes = formData.timePlayedMinutes ?? 0;
    newLogData.title = igdbGame.name;
    newLogData.date = formData.logDate;
    newLogData.rating = formData.rating;
    newLogData.finished = formData.finished;
    newLogData.status = formData.status;
    newLogData.notes = formData.notes ?? '';
    newLogData.timePlayed = timePlayed;
    newLogData.gameId = igdbGame.id;

    return newLogData;
}
