<template>
	<div class="p-4">
		<Setting v-model="config.key"></Setting>
		<Browser v-model="config.browser"></Browser>
	</div>
</template>

<script setup lang="ts">
	import Setting from '@/components/setting.vue';
	import { config } from '@/provider';
	import Browser from '@/components/browser.vue';
</script>
