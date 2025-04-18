<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { ref, onMounted, watch } from "vue";
import { ChevronDown, ChevronUp } from "lucide-vue-next";

interface TaskSession {
  id: string;
  duration: number;
  task_id: string;
  started: string;
  ended: string | null;
}

const props = defineProps<{
  taskId: string;
  isOpen: boolean;
}>();

const sessions = ref<TaskSession[]>([]);
const isExpanded = ref(props.isOpen);
const loading = ref(false);
const error = ref<string | null>(null);

async function fetchSessions() {
  if (!props.taskId) return;
  
  loading.value = true;
  error.value = null;
  
  try {
    sessions.value = await invoke("get_task_sessions", { taskId: props.taskId });
  } catch (err) {
    console.error("Failed to fetch sessions:", err);
    error.value = err as string;
  } finally {
    loading.value = false;
  }
}

function toggleExpand() {
  isExpanded.value = !isExpanded.value;
  
  if (isExpanded.value && sessions.value.length === 0) {
    fetchSessions();
  }
}

// Format date to readable format
function formatDate(dateString: string) {
  if (!dateString) return "";
  const date = new Date(dateString);
  return date.toLocaleString();
}

// Calculate duration between start and end times
function calculateDuration(start: string, end: string | null) {
  if (!end) return "In progress";
  
  const startDate = new Date(start);
  const endDate = new Date(end);
  const durationSeconds = Math.floor((endDate.getTime() - startDate.getTime()) / 1000);
  
  return `${durationSeconds} seconds`;
}

// Re-fetch when taskId changes
watch(() => props.taskId, () => {
  if (isExpanded.value) {
    fetchSessions();
  }
});

// Fetch on mount if expanded
onMounted(() => {
  if (isExpanded.value) {
    fetchSessions();
  }
});
</script>

<template>
  <div class="sessions-timeline w-full mt-2 border-t border-neutral-700 pt-2">
    <div 
      @click="toggleExpand"
      class="cursor-pointer flex items-center text-xs text-neutral-400 hover:text-white transition-colors">
      <span>Session Timeline</span>
      <ChevronDown v-if="!isExpanded" class="size-3 ml-1" />
      <ChevronUp v-else class="size-3 ml-1" />
    </div>
    
    <div v-if="isExpanded" class="mt-2">
      <div v-if="loading" class="text-xs text-neutral-400">Loading sessions...</div>
      
      <div v-else-if="error" class="text-xs text-red-400">Error: {{ error }}</div>
      
      <div v-else-if="sessions.length === 0" class="text-xs text-neutral-400">
        No sessions found for this task.
      </div>
      
      <div v-else class="space-y-2">
        <div 
          v-for="session in sessions" 
          :key="session.id"
          class="text-xs p-2 rounded bg-neutral-700/30 border border-neutral-700"
        >
          <div class="flex justify-between mb-1">
            <span class="font-medium">Started:</span>
            <span>{{ formatDate(session.started) }}</span>
          </div>
          
          <div class="flex justify-between mb-1">
            <span class="font-medium">Ended:</span>
            <span>{{ session.ended ? formatDate(session.ended) : 'In progress' }}</span>
          </div>
          
          <div class="flex justify-between">
            <span class="font-medium">Duration:</span>
            <span>{{ calculateDuration(session.started, session.ended) }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.sessions-timeline {
  transition: all 0.2s ease;
}
</style>