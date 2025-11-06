<script lang="ts">
	import { Badge } from '$lib/components/ui/badge';
	import { Separator } from '$lib/components/ui/separator';
	import { Progress } from '$lib/components/ui/progress';
	import * as Card from '$lib/components/ui/card';
	import { ScrollArea, Scrollbar } from '$lib/components/ui/scroll-area';
	import { ArrowLeftIcon, Clock, Star } from 'lucide-svelte';
	import { toTitleCase } from '$lib';
	import Button from '$lib/components/ui/button/button.svelte';

	export let data;
	const size = 200;
	const strokeWidth = 12;
	const score = data.game?.total_rating ? data.game.total_rating / 10 / 2 : 0;
	const comparisonScore = data.logStats?.averageRating ?? 0;
	const maxScore = 5;
	const radius = (size - strokeWidth) / 2;
	const center = size / 2;
	const circumference = 2 * Math.PI * radius;

	const scoreProgress = score !== undefined ? (score / maxScore) * circumference : 0;
	const comparisonProgress =
		comparisonScore !== undefined ? (comparisonScore / maxScore) * circumference : 0;

	const rotation = -90;
	const backgroundImageId = data.game?.artworks?.at(0);
	const totalTimePlayed = (data.logStats?.totalTimePlayed ?? 0) / 60;
	const timeToBeat = ((data.game?.time_to_beat?.normally ?? 0) / 3600).toFixed(1);
</script>

{#if data.game}
	<div class="min-h-full pb-16 relative">
		<Button
			variant="ghost"
			size="icon"
			class="absolute z-50 left-8 top-8"
			on:click={() => window.history.back()}
			><ArrowLeftIcon size={32} />
		</Button>
		<div class="relative h-[50vh] mb-12 from-primary to-transparent bg-gradient-to-b">
			{#if backgroundImageId}
				<img
					class="w-full h-full object-cover brightness-50 absolute top-0 left-0"
					alt=""
					src={`https://images.igdb.com/igdb/image/upload/t_1080p/${backgroundImageId}.jpg`}
				/>
			{/if}
			<div class="absolute h-full w-full pb-12">
				<div class="container flex items-end h-full justify-between">
					<div class="flex gap-4 items-end">
						{#if data.game.cover_image_id}
							<img
								src={'https://images.igdb.com/igdb/image/upload/t_cover_big/' +
									data.game.cover_image_id +
									'.jpg'}
								alt={data.game.title}
								class="w-32"
							/>
						{/if}
						<div>
							<h1 class="text-5xl font-heading line-clamp-2">{data.game.title}</h1>
							<div class="flex gap-2 items-center">
								<p>{data.game.release_date?.getFullYear() ?? 'Unknown'}</p>
								<p>•</p>
								<p>{data.game.developer ?? 'Unknown'}</p>
								<p>•</p>
								<div class="flex gap-1 cursor-default">
									{#each data.game.genres.slice(0, 7) ?? [] as genre}
										<Badge>{genre}</Badge>
									{/each}
								</div>
								{#if data.game.genres.length > 6}
									<span>+{data.game.genres.length - 6} more</span>
								{/if}
							</div>
						</div>
					</div>
					<div class={'relative inline-flex items-center justify-center'}>
						<svg width={size} height={size} class="transform -rotate-90">
							<circle
								cx={center}
								cy={center}
								r={radius}
								fill="none"
								stroke="hsl(var(--muted))"
								stroke-width={strokeWidth}
								class="opacity-20"
							/>
							<circle
								cx={center}
								cy={center}
								r={radius}
								fill="none"
								stroke="hsl(var(--primary))"
								stroke-width={strokeWidth}
								stroke-dasharray={circumference}
								stroke-dashoffset={circumference - comparisonProgress}
								stroke-linecap="round"
								style={`transform: rotate(${rotation}deg),
									transformOrigin: center`}
								class="transition-all duration-1000 ease-out"
							/>
							<circle
								cx={center}
								cy={center}
								r={radius}
								fill="none"
								stroke="yellow"
								stroke-width={strokeWidth}
								stroke-dasharray={circumference}
								stroke-dashoffset={circumference - scoreProgress}
								stroke-linecap="round"
								style={`transform: rotate(${rotation}deg),
									transformOrigin: center`}
								class="transition-all duration-1000 ease-out"
							/>
						</svg>
						<div class="absolute inset-0 flex flex-col items-center justify-center text-center">
							<span class="text-4xl font-bold" aria-label={`Score: ${score?.toFixed(1) || 'N/A'}`}>
								{score !== undefined ? score.toFixed(1) : 'N/A'}
							</span>
							<span
								class="text-sm"
								aria-label={`Your score: ${comparisonScore?.toFixed(1) || 'N/A'}`}
							>
								Your avg. rating: {comparisonScore !== undefined
									? comparisonScore.toFixed(1)
									: 'N/A'}
							</span>
						</div>
					</div>
				</div>
			</div>
		</div>
		<div class="container flex gap-4">
			<div class="flex flex-col gap-4 w-2/3">
				<Card.Root>
					<Card.Header>
						<Card.Title>About</Card.Title>
					</Card.Header>
					<Card.Content>
						<p class="mb-8 line-clamp-4">{data.game.description}</p>
						<div class="flex justify-between">
							<div class="flex-1">
								<h3 class="text-lg font-heading">Platforms</h3>
								<div class="flex gap-2 flex-wrap">
									{#if data.game.platforms.length > 0}
										{#each data.game.platforms as platform}
											<Badge>{platform}</Badge>
										{/each}
									{:else}
										<p>Unknown</p>
									{/if}
								</div>
							</div>
							<div class="flex-1">
								<h3 class="text-lg font-heading">Developer</h3>
								<p>{data.game.developer ?? 'Unknown'}</p>
							</div>
							<div class="flex-1">
								<h3 class="text-lg font-heading">Publisher</h3>
								<p>{data.game.publisher ?? 'Unknown'}</p>
							</div>
						</div>
					</Card.Content>
				</Card.Root>
				{#if data.game.screenshots.length > 0}
					<Card.Root>
						<Card.Header>
							<Card.Title>Screenshots</Card.Title>
						</Card.Header>
						<Card.Content>
							<ScrollArea>
								<div class="flex space-x-4 p-1 mb-2">
									{#each data.game.screenshots as screenshot}
										<img
											src={`https://images.igdb.com/igdb/image/upload/t_screenshot_big/${screenshot}.jpg`}
											class="rounded-lg w-96 aspect-video object-cover flex-shrink-0"
											loading="lazy"
											alt=""
										/>
									{/each}
								</div>
								<Scrollbar orientation="horizontal" />
							</ScrollArea>
						</Card.Content>
					</Card.Root>
				{/if}
				{#if data.game.videos.length > 0}
					<Card.Root>
						<Card.Header>
							<Card.Title>Videos</Card.Title>
						</Card.Header>
						<Card.Content>
							<ScrollArea>
								<div class="flex space-x-4 p-1 mb-2">
									{#each data.game.videos as video}
										<iframe
											src={`https://www.youtube.com/embed/${video}`}
											class="rounded-lg w-96 aspect-video object-cover flex-shrink-0"
											frameborder="0"
											title={`YouTube video for ${data.game.title}`}
											allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
											allowfullscreen
										></iframe>
									{/each}
								</div>
								<Scrollbar orientation="horizontal" />
							</ScrollArea>
						</Card.Content>
					</Card.Root>
				{/if}
				{#if data.logs?.length ?? 0 > 0}
					<Card.Root>
						<Card.Header>
							<Card.Title>Your Logs</Card.Title>
						</Card.Header>
						<Card.Content>
							<ScrollArea>
								<div class="flex flex-col gap-4">
									<Separator />
									{#each data.logs ?? [] as log, i}
										<div>
											<div class="flex justify-between items-center">
												<h3 class="text-lg font-heading mb-2">
													{toTitleCase(log.status)} on {new Date(
														log.startDate
													).toLocaleDateString()}
												</h3>
												<div class="flex gap-2">
													<div class="flex gap-1 items-center">
														<Clock size={16} />
														<p>{Math.floor(log.minutesPlayed / 60)}h {log.minutesPlayed % 60}m</p>
													</div>
													<div class="flex gap-1 items-center">
														<Star size={16} />
														<p>{log.rating}</p>
													</div>
												</div>
											</div>
											{#if log.notes}
												<p>{log.notes}</p>
											{:else}
												<p class="text-muted-foreground">No notes</p>
											{/if}
										</div>
										{#if i < (data.logs?.length ?? 0) - 1}
											<Separator />
										{/if}
									{/each}
								</div></ScrollArea
							>
						</Card.Content>
					</Card.Root>
				{/if}
			</div>
			<div class="flex flex-col gap-4 flex-1">
				<Card.Root>
					<Card.Header>
						<Card.Title>Your Stats</Card.Title>
					</Card.Header>
					<Card.Content>
						<div class="space-y-4">
							<div class="flex justify-between">
								<p>Time Played</p>
								<p>{totalTimePlayed} {totalTimePlayed === 1 ? 'hour' : 'hours'}</p>
							</div>
							<Separator />
							<div class="flex justify-between">
								<p>Times Logged</p>
								<p>{data.logStats?.count} {data.logStats?.count === 1 ? 'time' : 'times'}</p>
							</div>
							<Separator />
							<div class="flex justify-between">
								<p>Last Played</p>
								<p>
									{data.logStats?.lastPlayed
										? new Date(data.logStats.lastPlayed).toLocaleDateString()
										: 'Never'}
								</p>
							</div>
							{#if data.game.time_to_beat?.normally}
								<Separator />
								<div>
									<p class="mb-2">Completion Progress</p>
									<Progress value={totalTimePlayed} max={parseFloat(timeToBeat)} class="mb-1" />
									<div class="text-sm text-muted-foreground flex justify-between">
										<p>Current: {totalTimePlayed}h</p>
										<p>Average: {timeToBeat}h</p>
									</div>
								</div>
							{/if}
						</div>
					</Card.Content>
				</Card.Root>
			</div>
		</div>
	</div>
{/if}
