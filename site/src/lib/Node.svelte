<script lang="ts">
	export let state: GameState;
	export let selected: boolean;
</script>

<div
	class="p-2 pt-1 border-b-2 border-gray-700 
    {selected ? 'bg-gray-700' : 'bg-gray-900 hover:bg-gray-800'} "
	on:click
>
	<!-- Probability to 2DP -->
	<span class="inline-block w-full text-yellow-100 text-sm text-center font-extralight">
		{(state.probability * 100).toFixed(2)}% prob.
	</span>

	<ul class="flex flex-col items-center">
		{#each state.players as player, i}
			<li class="flex justify-between items-center mb-1 w-48 pr-1 rounded border-blue-300">
				<span
					class="inline-block w-7 text-center text-sm font-bold
                    text-gray-900 rounded-sm
                    {i === state.board.currentPlayer ? 'bg-gray-300' : 'bg-gray-400'}"
					class:in-jail={player.inJail}
				>
					{player.position}
				</span>

				<span class="text-sm">
					<span class="text-yellow-100 font-light">
						{player.doublesRolled}
					</span>
					dbls
				</span>

				<span class="text-sm text-yellow-200">
					${player.balance.toFixed(2)}
				</span>
			</li>
		{/each}
	</ul>
</div>

<style>
	.in-jail {
		box-shadow: inset 0px 0px 0px 2px rgb(254, 243, 201);
	}
</style>
