<script lang="ts">
	import '../app.css';
	import { GameState, PlayerFactory } from '$lib/minimax';
	import Node from '$lib/Node.svelte';

	// Initialise players
	const playerCount = 3;
	const players: Player[] = Array(playerCount);
	for (let i = 0; i < playerCount; i++) {
		players[i] = PlayerFactory();
	}

	// Initialise game
	let game = new GameState(players, {
		tiles: [],
		currentPlayer: 0,
		moveIsChance: true
	});

	let selectedNodes: number[] = [];
	let levels: GameState[][] = [game.getChildren()];

	function openTree(levelIndex: number, nodeIndex: number) {
		console.log(`Opening tree from (${levelIndex}, ${nodeIndex})`);

		while (levels.length - 1 > levelIndex) {
			levels.pop();
			selectedNodes.pop();
		}

		levels = [...levels, levels[levelIndex][nodeIndex].getChildren()];
		selectedNodes = [...selectedNodes, nodeIndex];
		console.log(levels);
	}
</script>

<svelte:head>
	<title>Monopoly Math</title>
</svelte:head>

<h1
	class="fixed w-full top-0 bg-gray-800 text-xl text-gray-200 text-center font-bold py-1 border-b-2 border-gray-500"
>
	Monopoly Math
</h1>

<main class="h-full overflow-y-hidden">
	{#each levels as nodes, l}
		<ul class="h-full overflow-y-scroll border-r-2 border-gray-600 inline-block pt-10">
			{#each nodes as child, n}
				<li>
					<Node state={child} selected={selectedNodes[l] === n} on:click={() => openTree(l, n)} />
				</li>
			{:else}
				No nodes here
			{/each}
		</ul>
	{/each}
</main>
