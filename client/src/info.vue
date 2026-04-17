<template>
	<div class="info">
		<h3>使用说明:</h3>
		<p>通过以下 URL 调用:</p>
		<code>http://localhost:{{ port }}/open?b=[浏览器路径]&url=[链接]</code>
		<p>请求头鉴权:</p>
		<code>x-openinbrowser-auth: base64url(nonce||ciphertext)</code>
	</div>
</template>
<script setup lang="ts">
	defineProps<{ port: number }>()
</script>