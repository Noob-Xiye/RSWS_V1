import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import ResourceCard from '../../components/ResourceCard';
import { Resource } from '../../types';

const mockResource: Resource = {
  id: '1',
  title: '测试资源',
  description: '这是一个测试资源',
  category: 'software',
  tags: ['test', 'demo'],
  price: 9.99,
  currency: 'USD',
  fileUrl: 'https://example.com/file.zip',
  thumbnailUrl: 'https://example.com/thumb.jpg',
  downloadCount: 100,
  rating: 4.5,
  authorId: 'author1',
  author: {
    id: 'author1',
    email: 'author@example.com',
    username: 'testauthor',
    avatar: 'https://example.com/avatar.jpg',
    createdAt: '2023-01-01T00:00:00Z',
    updatedAt: '2023-01-01T00:00:00Z',
  },
  createdAt: '2023-01-01T00:00:00Z',
  updatedAt: '2023-01-01T00:00:00Z',
};

describe('ResourceCard', () => {
  const mockOnView = jest.fn();
  const mockOnDownload = jest.fn();
  const mockOnFavorite = jest.fn();
  
  beforeEach(() => {
    jest.clearAllMocks();
  });
  
  it('renders resource information correctly', () => {
    render(
      <ResourceCard
        resource={mockResource}
        onView={mockOnView}
        onDownload={mockOnDownload}
        onFavorite={mockOnFavorite}
      />
    );
    
    expect(screen.getByText('测试资源')).toBeInTheDocument();
    expect(screen.getByText('这是一个测试资源')).toBeInTheDocument();
    expect(screen.getByText('$9.99')).toBeInTheDocument();
    expect(screen.getByText('software')).toBeInTheDocument();
    expect(screen.getByText('test')).toBeInTheDocument();
    expect(screen.getByText('demo')).toBeInTheDocument();
  });
  
  it('calls onView when view button is clicked', () => {
    render(
      <ResourceCard
        resource={mockResource}
        onView={mockOnView}
        onDownload={mockOnDownload}
        onFavorite={mockOnFavorite}
      />
    );
    
    fireEvent.click(screen.getByText('查看'));
    expect(mockOnView).toHaveBeenCalledWith(mockResource);
  });
  
  it('calls onDownload when download button is clicked', () => {
    render(
      <ResourceCard
        resource={mockResource}
        onView={mockOnView}
        onDownload={mockOnDownload}
        onFavorite={mockOnFavorite}
      />
    );
    
    fireEvent.click(screen.getByText('下载'));
    expect(mockOnDownload).toHaveBeenCalledWith(mockResource);
  });
  
  it('calls onFavorite when favorite button is clicked', () => {
    render(
      <ResourceCard
        resource={mockResource}
        onView={mockOnView}
        onDownload={mockOnDownload}
        onFavorite={mockOnFavorite}
      />
    );
    
    fireEvent.click(screen.getByText('收藏'));
    expect(mockOnFavorite).toHaveBeenCalledWith(mockResource);
  });
});