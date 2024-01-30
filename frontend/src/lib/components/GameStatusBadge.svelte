<script lang="ts">
	export let status: string;

	let statusColor: string;

	switch (status) {
		case 'Wishlist':
			statusColor = 'badge-secondary';
			break;
		case 'Backlog':
			statusColor = 'badge-info';
			break;
		case 'Playing':
			statusColor = 'badge-primary';
			break;
		case 'Completed':
			statusColor = 'badge-success';
			break;
		case 'Abandoned':
			statusColor = 'badge-error';
			break;
		case 'Shelved':
			statusColor = 'badge-warning';
			break;
		default:
			statusColor = 'badge-neutral';
			break;
	}
</script>

<span class="badge {statusColor}">{status}</span>
