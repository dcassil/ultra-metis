import { useState, useEffect, useCallback } from 'react'
import { useNavigate, Link } from 'react-router-dom'
import { listMachines, getMachine } from '../api/machines'
import type { Machine, MachineDetail } from '../api/machines'
import { createSession } from '../api/sessions'
import type { CreateSessionRequest } from '../api/sessions'
import { Button } from '../components/ui/Button'
import { FormInput } from '../components/ui/FormInput'
import { Select } from '../components/ui/Select'
import { Card } from '../components/ui/Card'
import { Badge } from '../components/ui/Badge'

type AutonomyLevel = 'normal' | 'stricter' | 'autonomous'

const autonomyOptions: Array<{
  value: AutonomyLevel
  label: string
  description: string
}> = [
  {
    value: 'normal',
    label: 'Normal',
    description: 'Standard autonomy — pauses for approval on significant actions.',
  },
  {
    value: 'stricter',
    label: 'Stricter',
    description: 'More cautious — pauses more frequently for human review.',
  },
  {
    value: 'autonomous',
    label: 'Autonomous',
    description: 'Minimal interruptions — proceeds independently where possible.',
  },
]

export default function NewSessionPage() {
  const navigate = useNavigate()

  // Machine & repo data
  const [machines, setMachines] = useState<Machine[]>([])
  const [machineDetail, setMachineDetail] = useState<MachineDetail | null>(null)
  const [loadingMachines, setLoadingMachines] = useState(true)
  const [loadingRepos, setLoadingRepos] = useState(false)

  // Form state
  const [machineId, setMachineId] = useState('')
  const [repoPath, setRepoPath] = useState('')
  const [title, setTitle] = useState('')
  const [instructions, setInstructions] = useState('')
  const [autonomyLevel, setAutonomyLevel] = useState<AutonomyLevel>('normal')
  const [workItemId, setWorkItemId] = useState('')
  const [context, setContext] = useState('')

  // Submission state
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Fetch online, trusted machines
  const fetchMachines = useCallback(async () => {
    try {
      const data = await listMachines()
      const eligible = data.filter(
        (m) => m.status === 'trusted' && m.connectivity_status === 'online',
      )
      setMachines(eligible)
    } catch {
      setError('Failed to load machines')
    } finally {
      setLoadingMachines(false)
    }
  }, [])

  useEffect(() => {
    void fetchMachines()
  }, [fetchMachines])

  // When machine selection changes, fetch its detail (which includes repos)
  useEffect(() => {
    if (!machineId) {
      setMachineDetail(null)
      setRepoPath('')
      return
    }

    let cancelled = false
    setLoadingRepos(true)
    setRepoPath('')

    getMachine(machineId)
      .then((detail) => {
        if (!cancelled) {
          setMachineDetail(detail)
        }
      })
      .catch(() => {
        if (!cancelled) {
          setError('Failed to load machine details')
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoadingRepos(false)
        }
      })

    return () => {
      cancelled = true
    }
  }, [machineId])

  const machineOptions = machines.map((m) => ({
    value: m.id,
    label: `${m.name} (${m.platform})`,
  }))

  const repoOptions = (machineDetail?.repos ?? []).map((r) => ({
    value: r.repo_path,
    label: `${r.repo_name} — ${r.repo_path}`,
  }))

  const canSubmit = machineId && repoPath && title.trim() && instructions.trim() && !submitting

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!canSubmit) return

    setSubmitting(true)
    setError(null)

    const payload: CreateSessionRequest = {
      machine_id: machineId,
      repo_path: repoPath,
      title: title.trim(),
      instructions: instructions.trim(),
      autonomy_level: autonomyLevel,
    }

    if (workItemId.trim()) {
      payload.work_item_id = workItemId.trim()
    }
    if (context.trim()) {
      payload.context = context.trim()
    }

    try {
      const result = await createSession(payload)
      navigate(`/sessions/${result.id}`)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create session')
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <div className="mx-auto max-w-2xl space-y-6">
      <div className="flex items-center gap-4">
        <Link to="/sessions" className="text-sm text-primary-600 hover:text-primary-800">
          &larr; Back to Sessions
        </Link>
      </div>

      <h1 className="text-2xl font-semibold text-secondary-900">Start New Session</h1>

      {error && (
        <div className="rounded-md bg-danger-50 border border-danger-200 p-3 text-sm text-danger-700">
          {error}
        </div>
      )}

      <form onSubmit={(e) => void handleSubmit(e)} className="space-y-6">
        {/* Machine & Repo Selection */}
        <Card title="Target">
          <div className="space-y-4">
            {loadingMachines ? (
              <p className="text-sm text-secondary-500">Loading machines...</p>
            ) : machines.length === 0 ? (
              <p className="text-sm text-secondary-500">
                No trusted, online machines available. Register and approve a machine first.
              </p>
            ) : (
              <Select
                label="Machine"
                options={machineOptions}
                value={machineId}
                onChange={setMachineId}
                placeholder="Select a machine..."
              />
            )}

            {machineId && (
              loadingRepos ? (
                <p className="text-sm text-secondary-500">Loading repositories...</p>
              ) : repoOptions.length === 0 ? (
                <p className="text-sm text-secondary-500">
                  No repositories found on this machine.
                </p>
              ) : (
                <Select
                  label="Repository"
                  options={repoOptions}
                  value={repoPath}
                  onChange={setRepoPath}
                  placeholder="Select a repository..."
                />
              )
            )}
          </div>
        </Card>

        {/* Session Details */}
        <Card title="Session Details">
          <div className="space-y-4">
            <FormInput
              label="Title"
              required
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="Brief description of what this session should accomplish"
            />

            <div>
              <label
                htmlFor="instructions"
                className="block text-sm font-medium text-secondary-700"
              >
                Instructions
                <span className="ml-0.5 text-danger-500">*</span>
              </label>
              <textarea
                id="instructions"
                required
                rows={5}
                value={instructions}
                onChange={(e) => setInstructions(e.target.value)}
                placeholder="Detailed instructions for the session..."
                className="mt-1 block w-full rounded-md border border-secondary-300 px-3 py-2 text-sm shadow-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
              />
            </div>
          </div>
        </Card>

        {/* Autonomy Level */}
        <Card title="Autonomy Level">
          <div className="grid gap-3 sm:grid-cols-3">
            {autonomyOptions.map((option) => {
              const isSelected = autonomyLevel === option.value
              return (
                <button
                  key={option.value}
                  type="button"
                  onClick={() => setAutonomyLevel(option.value)}
                  className={`rounded-lg border-2 p-3 text-left transition-colors ${
                    isSelected
                      ? 'border-primary-500 bg-primary-50'
                      : 'border-secondary-200 bg-white hover:border-secondary-300'
                  }`}
                >
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-medium text-secondary-900">{option.label}</span>
                    {isSelected && <Badge variant="info">Selected</Badge>}
                  </div>
                  <p className="mt-1 text-xs text-secondary-500">{option.description}</p>
                </button>
              )
            })}
          </div>
        </Card>

        {/* Optional Fields */}
        <Card title="Optional">
          <div className="space-y-4">
            <FormInput
              label="Work Item ID"
              value={workItemId}
              onChange={(e) => setWorkItemId(e.target.value)}
              placeholder="e.g. SMET-T-0123"
            />

            <div>
              <label
                htmlFor="context"
                className="block text-sm font-medium text-secondary-700"
              >
                Context
              </label>
              <textarea
                id="context"
                rows={3}
                value={context}
                onChange={(e) => setContext(e.target.value)}
                placeholder="Additional context for the session..."
                className="mt-1 block w-full rounded-md border border-secondary-300 px-3 py-2 text-sm shadow-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
              />
            </div>
          </div>
        </Card>

        {/* Submit */}
        <div className="flex justify-end">
          <Button type="submit" loading={submitting} disabled={!canSubmit}>
            Start Session
          </Button>
        </div>
      </form>
    </div>
  )
}
