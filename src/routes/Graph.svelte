<script lang="js">
    export let equation = '';

	// @ts-ignore
	import Desmos from 'desmos';
    import { onMount } from 'svelte';

	const elt = document.createElement('div');
	elt.style.width = '600px';
	elt.style.height = '400px';
	const calculator = Desmos.GraphingCalculator(elt, {keypad: false, expressions: false, settingsMenu: false});
    onMount(() => {
        const container = document.getElementById('calculator');
        if (container) {
            container.appendChild(elt);
        } else {
            console.error('Unable to mount Graph');
        }
    });

    $: if (equation !== '') {
        calculator.setExpression({ id: 'graph1', latex: equation });
    }; 
</script>

<div id="calculator">
	<!-- Calculator Goes here -->
</div>

