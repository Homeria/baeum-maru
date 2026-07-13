import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'

import App from './App'

describe('launcher app', () => {
  it('starts with the server stopped', () => {
    render(<App />)

    expect(screen.getByText('서버 중지')).toBeInTheDocument()
  })
})
