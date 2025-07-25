package models

import "gorm.io/gorm"

type Resource struct {
	gorm.Model
	Title     string `json:"title" gorm:"not null;size:200"`
	Content   string `json:"content" gorm:"type:text"`
	Language  string `json:"language" gorm:"size:10"`
	Format    string `json:"format" gorm:"size:50"` // pdf, video, audio, text, etc.
	CreatorID uint   `json:"creator_id" gorm:"not null;index"`
	
	// Foreign key relationship
	Creator User `json:"creator" gorm:"foreignKey:CreatorID;constraint:OnUpdate:CASCADE,OnDelete:CASCADE"`
}

// TableName specifies the table name for Resource model
func (Resource) TableName() string {
	return "resources"
}