<script lang="ts">
	import {
		CalendarDate,
		DateFormatter,
		getLocalTimeZone,
		parseDate
	} from '@internationalized/date';
	import { cn } from '$lib/utils.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Calendar } from '$lib/components/ui/calendar/index.js';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import { CalendarIcon } from 'lucide-svelte';

	const df = new DateFormatter('en-US', {
		dateStyle: 'long'
	});

	export let value: Date | undefined = undefined;
	export let minValue: CalendarDate | undefined = undefined;
	export let maxValue: CalendarDate | undefined = undefined;
	export let disabled = false;
</script>

<Popover.Root>
	<Popover.Trigger {...$$restProps} asChild let:builder>
		<Button
			variant="outline"
			class={cn('w-full justify-start text-left font-normal', !value && 'text-muted-foreground')}
			builders={[builder]}
		>
			<CalendarIcon class="mr-2 h-4 w-4" />
			{value ? df.format(value) : 'Pick a date'}
		</Button>
	</Popover.Trigger>
	<Popover.Content class="w-auto p-0" align="start">
		<Calendar
			{disabled}
			value={value ? parseDate(value?.toISOString().substring(0, 10)) : undefined}
			{minValue}
			{maxValue}
			onValueChange={(newValue) => {
				value = newValue ? newValue.toDate(getLocalTimeZone()) : undefined;
			}}
		/>
	</Popover.Content>
</Popover.Root>
