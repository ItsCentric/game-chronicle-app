<script lang="ts">
    import { main } from "../wailsjs/go/models";

    export let gameLog: main.Log;
</script>

<div class="card bg-base-100 shadow-xl">
    <div class="card-body">
        <p class="card-title">{gameLog.title}</p>
        <p class="card-subtitle">{gameLog.notes}</p>
    </div>
</div>
