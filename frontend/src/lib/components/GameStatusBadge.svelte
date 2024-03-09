<script lang="ts">
	export let status: string;

	let statusColor: string;

	switch (status) {
		case 'Wishlist':
			statusColor = '';
			break;
		case 'Backlog':
			statusColor = '';
			break;
		case 'Playing':
			statusColor = '';
			break;
		case 'Completed':
			statusColor = '';
			break;
		case 'Abandoned':
			statusColor = '';
			break;
		case 'Shelved':
			statusColor = '';
			break;
		default:
			statusColor = '';
			break;
	}
</script>

<span class=" {statusColor}">{status}</span>
