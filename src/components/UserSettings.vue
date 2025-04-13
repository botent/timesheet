<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";

async function resetDb() {
    await invoke("reset_database").then(() => {
        alert("Data Deleted");
    });
}
</script>

<template>
    <button @click="resetDb">Delete All Data</button>
</template>
