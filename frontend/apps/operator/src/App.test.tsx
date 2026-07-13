import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'

import App from './App'

describe('operator app', () => {
  it('renders the application identity', () => {
    render(<App />)

    expect(screen.getByRole('heading', { name: '업무 웹' })).toBeInTheDocument()
  })
})
