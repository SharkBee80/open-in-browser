<template>
	<div class="info">
		<h3>使用说明:</h3>
		<p>通过以下 URL 调用:</p>
		<code>http://localhost:{{ port }}/cmd?b=[浏览器路径]&url=[链接]&args=[可选参数]</code>
		<p>请求头鉴权 (JWT):</p>
		<code>x-openinbrowser-auth: [JWT Token]</code>
		<p class="note">注: 使用 HS256 算法,密钥为设置的鉴权 Key</p>
	</div>
</template>
<script setup lang="ts">
	defineProps<{ port: number }>()
</script>
<style scoped>
	.note {
		font-size: 0.85rem;
		color: #666;
		margin-top: 0.5rem;
		font-style: italic;
	}
</style>