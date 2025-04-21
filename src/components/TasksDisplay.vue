<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { ref } from "vue";
import { CheckIcon, Delete, Play, Pause, Clock } from "lucide-vue-next";
import { Task, TaskStatus } from "../types";
import TaskSessionsTimeline from "./TaskSessionsTimeline.vue";

const allTasks = ref<{ [date: string]: [Task, number][] }>({});
const expandedTaskIds = ref<Set<string>>(new Set());

async function fetchAndDisplayTasks() {
    try {
        allTasks.value = await invoke("display_tasks");
        console.log("Tasks:", allTasks.value);
    } catch (error) {
        console.error("Failed to fetch tasks:", error);
    }
}

async function updateTaskStatus(taskId: string, status: TaskStatus) {
    try {
        await invoke("update_task_status", { taskId, status });
        console.log(`Task ${taskId} status updated to ${status}`);
        fetchAndDisplayTasks(); // Refresh tasks after update
    } catch (error) {
        console.error(`Failed to update task status: ${error}`);
    }
}

async function deleteTask(taskId: string) {
    try {
        await invoke("delete_task", { taskId });
        console.log(`Task ${taskId} deleted successfully`);
        fetchAndDisplayTasks(); // Refresh tasks after deletion
    } catch (error) {
        console.error(`Failed to delete task: ${error}`);
    }
}

function toggleTaskExpansion(taskId: string) {
    if (expandedTaskIds.value.has(taskId)) {
        expandedTaskIds.value.delete(taskId);
    } else {
        expandedTaskIds.value.add(taskId);
    }
}

fetchAndDisplayTasks();
</script>

<template>
    <section
        class="w-full min-w-[60%] mx-auto px-6 py-2 rounded-md bg-neutral-800 border border-neutral-700"
    >
        <div
            v-for="(val, key) in allTasks"
            :key="key"
            class="border-b border-neutral-700 py-4"
        >
            <p class="text-xs font-semibold text-start mb-2">
                {{ key }}
            </p>
            <div
                v-for="([task], index) in val"
                :key="index"
                class="w-full mb-4"
            >
                <div
                    class="flex flex-row items-center justify-between space-x-4 mb-2"
                >
                    <div class="flex items-center space-x-2 w-1/2">
                        <p class="font-semibold">{{ task.title }}</p>
                        <button
                            class="rounded-full p-1 hover:bg-neutral-700 transition-colors"
                            @click="toggleTaskExpansion(task.id)"
                            title="View Session Timeline"
                        >
                            <Clock class="size-3 text-neutral-400" />
                        </button>
                    </div>
                    <p
                        class="text-xs rounded-md p-1 bg-teal-500/10 border border-teal-600/10"
                    >
                        {{ task.status }}
                    </p>

                    <div
                        class="place-self-end flex flex-row items-center justify-end space-x-2"
                    >
                        <button
                            v-if="task.status !== TaskStatus.RUNNING"
                            class="rounded-sm !p-1 border border-neutral-700"
                            @click="
                                updateTaskStatus(task.id, TaskStatus.RUNNING)
                            "
                        >
                            <Play class="size-3" />
                        </button>
                        <button
                            v-if="task.status === TaskStatus.RUNNING"
                            class="rounded-sm !p-1 border border-neutral-700"
                            @click="
                                updateTaskStatus(task.id, TaskStatus.PAUSED)
                            "
                        >
                            <Pause class="size-3" />
                        </button>
                        <button
                            class="rounded-sm !p-1 border border-neutral-700"
                            @click="
                                updateTaskStatus(task.id, TaskStatus.COMPLETED)
                            "
                        >
                            <CheckIcon class="size-3" />
                        </button>
                        <button
                            class="rounded-sm !p-1 border border-neutral-700"
                            @click="deleteTask(task.id)"
                        >
                            <Delete class="size-3" />
                        </button>
                    </div>
                </div>

                <!-- Session Timeline component -->
                <TaskSessionsTimeline
                    v-if="task.id"
                    :taskId="task.id"
                    :isOpen="expandedTaskIds.has(task.id)"
                />
            </div>
        </div>
    </section>
</template>

<style scoped></style>
