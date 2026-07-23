import { Alert, Button } from '@mantine/core'
import { useNavigate } from 'react-router-dom'

// 학기 업무 화면에서 학기가 선택되지 않았을 때의 안내.
export function TermNotice() {
  const navigate = useNavigate()
  return (
    <Alert color="blue" title="학기를 먼저 선택하세요">
      상단의 학기 선택기에서 학기를 고르면 이 화면이 그 학기 기준으로 표시됩니다. 등록된 학기가 없으면{' '}
      <Button variant="subtle" size="compact-xs" onClick={() => navigate('/terms')}>
        학기 관리
      </Button>
      에서 추가하세요.
    </Alert>
  )
}
