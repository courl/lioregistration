import { type MaybeRefOrGetter, ref, toValue, watch } from "vue"

import type { EditionId } from "@/types/edition.ts"
import type { EmailTemplateId } from "@/types/email"
import type {
  NewRound,
  NewTask,
  Round,
  RoundDetails,
  RoundId,
  TaskId,
  UpdateParticipant,
  UpdateRound,
  UpdateTask,
} from "@/types/round.ts"
import type { StudentId } from "@/types/student.ts"

import { api } from "@/composables/api.ts"

export const useRounds = (id: MaybeRefOrGetter<EditionId | undefined>) => {
  const loading = ref<boolean>(false)
  const items = ref<Round[]>([])

  const reload = async (id: EditionId | undefined) => {
    if (id === undefined) {
      items.value = []
      return
    }
    loading.value = true
    try {
      items.value = await api<Round[]>(
        `/admin/editions/${id.toString()}/rounds`,
      )
    } finally {
      loading.value = false
    }
  }

  const createRound = async (data: NewRound) => {
    const id_ = toValue(id)
    if (id_ === undefined) {
      return
    }
    await api(`/admin/editions/${id_.toString()}/rounds`, {
      method: "POST",
      body: data,
    })
    await reload(id_)
  }

  const updateRound = async (data: UpdateRound) => {
    const id_ = toValue(id)
    if (id_ === undefined) {
      return
    }
    await api(`/admin/rounds/${data.id.toString()}`, {
      method: "PATCH",
      body: { ...data, id: undefined },
    })
    await reload(id_)
  }

  const deleteRound = async (round: RoundId) => {
    await api(`/admin/rounds/${round.toString()}`, { method: "DELETE" })
    await reload(toValue(id))
  }

  const moveTo = async (round: RoundId, students: StudentId[]) => {
    await api(`/admin/rounds/${round.toString()}`, {
      method: "POST",
      body: students,
    })
    await reload(toValue(id))
  }

  watch(() => toValue(id), reload, { immediate: true })

  return {
    rounds: items,
    loading,
    createRound,
    deleteRound,
    updateRound,
    moveTo,
  }
}

export const useRound = (id: MaybeRefOrGetter<RoundId | undefined>) => {
  const loading = ref<boolean>(false)
  const item = ref<RoundDetails>()

  const reload = async (id: RoundId | undefined) => {
    if (id === undefined) {
      item.value = undefined
      return
    }
    loading.value = true
    try {
      item.value = await api<RoundDetails>(`/admin/rounds/${id.toString()}`)
    } finally {
      loading.value = false
    }
  }

  const createTask = async (data: NewTask) => {
    const roundId = toValue(id)
    if (roundId === undefined) {
      return
    }
    await api(`/admin/rounds/${roundId.toString()}`, {
      method: "POST",
      body: data,
    })
    await reload(roundId)
  }

  const updateTask = async (data: UpdateTask) => {
    const roundId = toValue(id)
    if (roundId === undefined) {
      return
    }
    await api(`/admin/tasks/${data.id.toString()}`, {
      method: "PATCH",
      body: { ...data, id: undefined },
    })
    await reload(roundId)
  }

  const deleteTask = async (taskId: TaskId) => {
    const roundId = toValue(id)
    if (roundId === undefined) {
      return
    }
    await api(`/admin/tasks/${taskId.toString()}`, { method: "DELETE" })
    await reload(roundId)
  }

  const updateParticipation = async (
    studentId: StudentId,
    data: UpdateParticipant,
  ) => {
    const roundId = toValue(id)
    if (roundId === undefined) {
      return
    }
    await api(
      `/admin/rounds/${roundId.toString()}/participations/${studentId.toString()}`,
      { method: "PATCH", body: data },
    )
    await reload(roundId)
  }

  const bulkUpdateScores = async (
    data: { credentials_username: string; task_scores: [TaskId, number][] }[],
  ) => {
    const roundId = toValue(id)
    if (roundId === undefined) {
      return
    }
    await api(`/admin/rounds/${roundId.toString()}/scores`, {
      method: "POST",
      body: data,
    })
    await reload(roundId)
  }

  const sendEmail = async (
    template_id: EmailTemplateId,
    headline: string,
    body: string,
  ) => {
    const roundId = toValue(id)
    if (roundId === undefined) {
      return
    }

    await api(`/admin/emails/${template_id.toString()}/send`, {
      method: "POST",
      body: { round_id: roundId, headline, body },
    })
  }

  watch(() => toValue(id), reload, { immediate: true })

  return {
    round: item,
    loading,
    createTask,
    updateTask,
    deleteTask,
    updateParticipation,
    bulkUpdateScores,
    sendEmail,
  }
}
