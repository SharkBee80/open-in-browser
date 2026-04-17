<template>
	<div class="flex justify-between items-center">
		<a href="/popup.html" :target="chromename + 'popup'">
			<h1 class="text-xl font-bold mb-4">Settings</h1>
		</a>
		<div class="flex items-center gap-2">
			<Button :icon="'pi pi-spin ' + (isDark ? 'pi-moon' : 'pi-sun')" rounded severity="contrast" size="small"
				@click="toggleDark()" />
			<Button icon="pi pi-refresh" size="small" severity="danger" @click="reset()" />
		</div>
	</div>
	<div class="p-fluid mb-4">
		<div class="p-field">
			<label for="port">Port</label>
			<InputNumber id="port" v-model="model.port" :useGrouping="false" showButtons :min="1023" :max="65535" />
		</div>
		<div class="p-field mt-4">
			<label for="secretKey">Secret Key</label>
			<InputText id="secretKey" v-model="model.secret" spellcheck="false" :placeholder="DEFAULT_KEY.secret" />
		</div>
	</div>
</template>
<script setup lang="ts">
	import { DEFAULT_KEY } from '@/provider';
	import { useDark, useToggle } from '@vueuse/core';
	const isDark = useDark();
	const toggleDark = useToggle(isDark)
	const model = defineModel<{ port: number, secret: string }>({ default: DEFAULT_KEY })
	const chromename = chrome.runtime.id + '_';
	const reset = () => {
		model.value = DEFAULT_KEY;
	}
</script>
<style scoped>
	.p-field {
		display: flex;
		gap: 1rem;
		align-items: center;
	}

	.p-field label {
		font-size: 1rem;
	}
</style>