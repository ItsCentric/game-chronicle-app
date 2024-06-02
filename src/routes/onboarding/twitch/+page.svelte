<script lang="ts">
	import Input from '$lib/components/ui/input/input.svelte';
	import type { PageData } from './$types';
	import { superForm } from 'sveltekit-superforms';
	import * as Form from '$lib/components/ui/form';
	import { zodClient } from 'sveltekit-superforms/adapters';
	import { twitchCredentialsSchema } from '$lib/schemas';
	import { ArrowLeft } from 'lucide-svelte';
	import { Button } from '$lib/components/ui/button';
	import { getUserSettings, saveUserSettings } from '$lib/rust-bindings/helpers';
	import { goto } from '$app/navigation';

	export let data: PageData;

	const twitchForm = superForm(data.form, {
		SPA: true,
		validators: zodClient(twitchCredentialsSchema),
		onUpdate: async ({ form }) => {
			if (!form.valid) return;
			const settings = await getUserSettings();
			settings.twitch_client_id = form.data.clientId;
			settings.twitch_client_secret = form.data.clientSecret;
			await saveUserSettings(settings);
			goto('/onboarding/game-detection');
		}
	});
	const { form: twitchFormData, enhance, validateForm } = twitchForm;
	let isFormValid = false;
	$: if ($twitchFormData)
		validateForm({ update: false }).then((form) => (isFormValid = form.valid));
</script>

<main class="py-16">
	<div class="max-w-prose mx-auto">
		<div class="relative mb-8">
			<Button
				href="/onboarding/import/steam"
				class="absolute top-2 -translate-x-full -left-4"
				variant="ghost"
				size="icon"><ArrowLeft size={48} /></Button
			>
			<h1 class="text-3xl font-heading font-bold">Set your Twitch credentials</h1>
			<p>
				Twitch credentials are required to retrieve information that Twitch provides about titles
				across the platform.
			</p>
		</div>
		<form method="post" use:enhance class="relative flex flex-col gap-16" id="twitchForm">
			<div class="absolute -translate-x-full -left-6">
				<div
					class="text-3xl flex justify-center items-center font-bold aspect-square h-auto w-16 text-center rounded-full border border-muted"
				>
					<p>1</p>
				</div>
				<div class="mx-auto h-40 bg-muted w-px"></div>
				<div
					class="text-3xl flex justify-center items-center font-bold aspect-square h-auto w-16 text-center rounded-full border border-muted"
				>
					<p>2</p>
				</div>
				<div class="mx-auto h-40 bg-muted w-px"></div>
				<div
					class="text-3xl flex justify-center items-center font-bold aspect-square h-auto w-16 text-center rounded-full border border-muted"
				>
					<p>3</p>
				</div>
			</div>
			<div>
				<h2 class="text-2xl font-heading font-bold">Register an application</h2>
				<p>
					Go to Twitch's <a
						href="https://dev.twitch.tv/console/apps/create"
						target="_blank"
						class="text-accent">application registration page</a
					>
					and create a new application. You can name it whatever you want, and you can use any website
					URL for the OAuth Redirect URL. You can use <code>http://localhost:3000</code> if you're not
					sure what to use. For the category, select "Application Integration". You can leave the Client
					Type as "Confidential" and click "Create".
				</p>
			</div>
			<div>
				<h2 class="text-2xl font-heading font-bold">Provide your Twitch Client ID</h2>
				<p class="mb-4">
					After creating your application, you will be taken to a page with your Client ID and
					secret. Copy and paste it here.
				</p>
				<Form.Field name="clientId" form={twitchForm}>
					<Form.Control let:attrs>
						<Input {...attrs} bind:value={$twitchFormData.clientId} placeholder="Client ID" />
					</Form.Control>
					<Form.FieldErrors />
				</Form.Field>
			</div>
			<div>
				<h2 class="text-2xl font-heading font-bold">Provide your Twitch Client Secret</h2>
				<p class="mb-4">
					Below your Client ID, you will be able to create a new secret. Click "New Secret" and copy
					and paste it here.
				</p>
				<Form.Field name="clientSecret" form={twitchForm}>
					<Form.Control let:attrs>
						<Input
							{...attrs}
							bind:value={$twitchFormData.clientSecret}
							placeholder="Client Secret"
						/>
					</Form.Control>
					<Form.FieldErrors />
				</Form.Field>
			</div>
		</form>
		<Form.Button disabled={!isFormValid} class="float-right mt-4" form="twitchForm"
			>Next</Form.Button
		>
	</div>
</main>
