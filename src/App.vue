<script setup lang="ts">
import TaskCreate from "./components/TaskCreate.vue";
import TasksDisplay from "./components/TasksDisplay.vue";
import UserSettings from "./components/UserSettings.vue";

import { ref } from "vue";

const reloads = ref(0);
const showTaskCreate = ref(false);
const showUserSettings = ref(false);
</script>

<template>
    <main
        class="max-w-[80%] mx-auto h-screen flex flex-col items-start justify-start p-4 space-y-4 bg-neutral-900 text-white"
    >
        <div class="w-full flex flex-row items-center justify-between">
            <button @click="showTaskCreate = true">Create a task</button>
            <button @click="showUserSettings = true">Settings</button>
        </div>
        <UserSettings v-if="showUserSettings" />
        <TaskCreate
            @task-creation-success="reloads += 1"
            v-if="showTaskCreate"
        />
        <TasksDisplay :key="reloads" />
    </main>
</template>
