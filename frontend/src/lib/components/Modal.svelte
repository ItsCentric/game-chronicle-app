<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { X } from 'lucide-svelte';

	export let open = false;

	let dialogElement: HTMLDialogElement | undefined;
	const openDispatcher = createEventDispatcher();

	$: if (open) {
		dialogElement?.showModal();
		openDispatcher('open', true);
	} else {
		dialogElement?.close();
	}
</script>

<dialog bind:this={dialogElement} on:close class="modal">
	<div class="modal-box container max-w-3xl">
		<div>
			<button class="float-right" on:click={() => (open = false)}>
				<X size={24} />
			</button>
		</div>
		<slot />
	</div>
	<form method="dialog" class="modal-backdrop">
		<button>close</button>
	</form>
</dialog>
