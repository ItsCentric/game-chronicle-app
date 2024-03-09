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

<dialog bind:this={dialogElement} on:close class="">
	<div class="container max-w-3xl">
		<div class="flex justify-between items-center mb-4">
			<div class="flex gap-2 items-center text-2xl font-semibold">
				<slot name="heading" />
			</div>
			<button on:click={() => (open = false)}>
				<X size={32} />
			</button>
		</div>
		<slot name="content" />
	</div>
</dialog>
