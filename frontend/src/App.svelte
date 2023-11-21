<script lang="ts">
    import { InsertGameLog, GetAllGameLogs } from "./lib/wailsjs/go/main/Database";
    import toast, { Toaster } from "svelte-french-toast";
    import { main } from "./lib/wailsjs/go/models";

    let formElement;
    const initialForm = {
        title: "",
        rating: 0,
        notes: "",
        status: "Completed",
        finished: false,
        timePlayedHours: 0,
        timePlayedMinutes: 0,
    };
    let form = initialForm;

    let getLogsPromise = GetAllGameLogs();
    async function handleSubmit() {
        const finished = form.finished ? 1 : 0;
        const newLogData = new main.LogData({ title: form.title, rating: form.rating, notes: form.notes, status: form.status, finished: finished, timePlayed: new main.TimePlayed({ hours: form.timePlayedHours, minutes: form.timePlayedMinutes })})
        try {
            await InsertGameLog(newLogData);
            toast.success("New log created!");
        } catch (e) {
            toast.error("Failed to create new log");
            return;
        }
        getLogsPromise = GetAllGameLogs();
        form = initialForm;
        formElement.reset();
    }
</script>

<main class="flex justify-center items-center h-full">
    <Toaster />
    <form bind:this={formElement}>
        <div class="form-control">
            <label for="title" class="label">
                <span class="label-text">Game Title</span>
            </label>
            <input
                id="title"
                type="text"
                placeholder="A cool title"
                bind:value={form.title}
                class="input input-bordered w-full max-w-xs"
            />
            <label for="rating" class="label">
                <span class="label-text">Rating</span>
            </label>
            <input
                id="rating"
                type="number"
                placeholder="1-10"
                min={1}
                max={10}
                bind:value={form.rating}
                class="input input-bordered w-full max-w-xs"
            />
            <label for="notes" class="label">
                <span class="label-text">Notes</span>
            </label>
            <textarea
                id="notes"
                placeholder="Some notes"
                bind:value={form.notes}
                class="textarea textarea-bordered w-full max-w-xs"
            />
            <label for="status" class="label">
                <span class="label-text">Status</span>
            </label>
            <select
                id="status"
                class="select select-bordered w-full max-w-xs"
                bind:value={form.status}
            >
                <option value="Backlog">Want to play</option>
                <option value="Playing">Playing</option>
                <option value="Completed">Played</option>
            </select>
            <label for="finished" class="label">
                <span class="label-text">Finished</span>
            </label>
            <input
                id="finished"
                type="checkbox"
                bind:checked={form.finished}
                class="checkbox checkbox-accent"
            />
            <label for="time-played" class="label">
                <span class="label-text">Time Played</span>
            </label>
            <input
                id="time-played-hours"
                type="number"
                placeholder="Hours"
                bind:value={form.timePlayed}
                min={0}
                class="input input-bordered w-full max-w-xs"
            />
            <input
                id="time-played-minutes"
                type="number"
                placeholder="Minutes"
                bind:value={form.timePlayedMinutes}
                min={0}
                max={59}
                class="input input-bordered w-full max-w-xs"
            />
            <input
                type="button"
                value="Submit"
                class="btn btn-primary"
                on:click={handleSubmit}
            />
        </div>
    </form>
    <div class="divider divider-horizontal" />
    <div class="grid grid-cols-3">
        {#await getLogsPromise}
            <span class="loader loader-lg" />
            <p>Getting logs...</p>
        {:then logs}
            {#each logs as log}
                <div class="card bg-base-100 shadow-xl">
                    <div class="card-body">
                        <p class="card-title">{log.title}</p>
                        <p class="card-subtitle">{log.notes}</p>
                    </div>
                </div>
            {/each}
        {/await}
    </div>
</main>
